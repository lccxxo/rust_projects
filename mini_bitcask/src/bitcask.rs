use fs4::FileExt;
use std::{
    collections::BTreeMap,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

const KEY_VAL_HEADER_LEN: u32 = 4; // 键值长度头的大小（4字节）
const MERGE_FILE_EXT: &str = "merge"; // 合并操作的别名

type KyeDir = BTreeMap<Vec<u8>, (u64, u32)>; // key: 数据的键 value: 值在文件中的偏移量，值的长度

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub struct MiniBitcask {
    log: Log,       // 日志文件
    keydir: KyeDir, // 内存索引
}

impl Drop for MiniBitcask {
    fn drop(&mut self) {}
}

impl MiniBitcask {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut log = Log::new(path)?;
        let keydir = log.load_index()?;
        Ok(Self { log, keydir })
    }

    pub fn merge(&mut self) -> Result<()> {
        let mut merge_path = self.log.path.clone();
        merge_path.set_extension(MERGE_FILE_EXT);

        let mut new_log = Log::new(merge_path)?;
        let mut new_keydir = KyeDir::new();

        // 将数据重新写回KeyDir
        for (key, (value_pos, value_len)) in self.keydir.iter() {
            let value = self.log.read_value(*value_pos, *value_len)?;
            let (offset, len) = new_log.write_entry(key, Some(&value))?;
            new_keydir.insert(
                key.clone(),
                (offset + len as u64 - *value_len as u64, *value_len),
            );
        }

        // 文件重命名
        std::fs::rename(new_log.path, self.log.path.clone())?;

        new_log.path = self.log.path.clone();
        self.log = new_log;
        self.keydir = new_keydir;

        Ok(())
    }

    pub fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<()>{
        let (offset, len) = self.log.write_entry(key, Some(&value))?;
        let value_len = value.len() as u32;

        self.keydir.insert(
            key.to_vec(), 
            (offset + len as u64 - value_len  as u64, value_len),
        );

        Ok(())
    }

    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>>{
        if let Some((value_pos, value_len)) = self.keydir.get(key){
            let val = self.log.read_value(*value_pos, *value_len)?;
            Ok(Some(val))
        }else {
            Ok(None)
        }
    }

    pub fn delete(&mut self, key:&[u8])-> Result<()>{
        self.log.write_entry(key, None)?;
        self.keydir.remove(key);
        Ok(())
    }

    // fn flush(&mut self)->Result<()>{
    //     Ok(self.log.file.sync_all()?)
    // }

}

struct Log {
    path: PathBuf,
    file: std::fs::File,
}

impl Log {
    fn new(path: PathBuf) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }

        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        // 加 exclusive lock 防止并发更新
        file.try_lock_exclusive()?;

        Ok(Self { path, file })
    }

    fn load_index(&mut self) -> Result<KyeDir> {
        let mut len_buf = [0u8; KEY_VAL_HEADER_LEN as usize]; // 4字节缓冲区 读取key_len value_len
        let mut keydir = KyeDir::new(); // 空索引
        let file_len = self.file.metadata()?.len(); // 文件总大小
        let mut r = BufReader::new(&mut self.file); // 文件读取
        let mut pos: u64 = r.seek(SeekFrom::Start(0))?; // 文件开头位置

        // 循环读取文件
        while pos < file_len {
            let read_one = || -> Result<(Vec<u8>,u64,Option<u32>)>{
                // 读取key_len
                r.read_exact(&mut len_buf)?;
                let key_len = u32::from_be_bytes(len_buf);
                // 读取value_len
                r.read_exact(&mut len_buf)?;
                let value_lent_or_tombstone = match i32::from_be_bytes(len_buf){
                    l if l >= 0 => Some(l as u32),
                    _ => None,
                };

                // value的位置
                let value_pos = pos + KEY_VAL_HEADER_LEN as u64 * 2 + key_len as u64;

                // 读取key的内容
                let mut key = vec![0; key_len as usize];
                r.read_exact(&mut key)?;

                // 跳过value的值
                // 详细解释下
                // value_len 始终是大于0的u32 如果大于0 则指针向前移动value_len 直接指到下一个kv的key_len处 如果不大于零 则不执行seek_relative
                if let Some(value_len) = value_lent_or_tombstone{
                    r.seek_relative(value_len as i64)?;
                }

                Ok((key, value_pos, value_lent_or_tombstone))
            }();


            match read_one {
                Ok((key, value_pos, Some(value_len))) => {
                    keydir.insert(key, (value_pos, value_len) );
                    pos = value_pos + value_len as u64;
                }
                Ok((key, value_pos, None)) => {
                    keydir.remove(&key);
                    pos = value_pos;
                }
                Err(err) => return Err(err),
            }
        }

        Ok(keydir)
    }

    // 根据 value 的位置和长度获取 value 的值
    fn read_value(&mut self, value_pos: u64, value_len: u32) -> Result<Vec<u8>> {
        let mut value = vec![0; value_len as usize];
        self.file.seek(SeekFrom::Start(value_pos))?;
        self.file.read_exact(&mut value)?;
        Ok(value)
    }

    // +-------------+-------------+----------------+----------------+
    // | key len(4)    val len(4)     key(varint)       val(varint)  |
    // +-------------+-------------+----------------+----------------+
    fn write_entry(&mut self, key: &[u8], value: Option<&[u8]>) -> Result<(u64, u32)> {
        let key_len = key.len() as u32;
        let val_len = value.map_or(0, |v| v.len() as u32);
        let val_len_tomestone = value.map_or(-1,|v| v.len() as i32);

        // 总长度
        let len = KEY_VAL_HEADER_LEN * 2 + key_len + val_len;

        let offset = self.file.seek(SeekFrom::End(0))?;
        
        let mut writer = BufWriter::with_capacity(len as usize, &self.file);
        writer.write_all(&key_len.to_be_bytes())?;
        writer.write_all(&val_len_tomestone.to_be_bytes())?;
        writer.write_all(key)?;
        if let Some(value) = value {
            writer.write_all(value)?;
        }
        writer.flush()?;
    
        Ok((offset, len))
    }
}



mod tests {

    use super::Log;

    #[test]
    fn test_log_reopen() -> crate::bitcask::Result<()> {
        let path = std::env::temp_dir()
            .join("sqldb-disk-engine-log-test2")
            .join("log");

        println!("测试文件路径: {:?}", path.clone());

        let mut log = Log::new(path.clone())?;
        log.write_entry(b"a", Some(b"val1"))?;
        log.write_entry(b"b", Some(b"val2"))?;
        log.write_entry(b"c", Some(b"val3"))?;
        log.write_entry(b"d", Some(b"val4"))?;
        log.write_entry(b"d", None)?;
        log.write_entry(b"c", None)?;

        // 不 drop，直接加载索引
        let keydir = log.load_index()?;
        println!("第一次加载索引，大小: {}", keydir.len());
        for (key, (pos, len)) in &keydir {
            println!("key: {:?}, pos: {}, len: {}", String::from_utf8_lossy(key), pos, len);
        }

        drop(log);
        std::thread::sleep(std::time::Duration::from_millis(100));

        let mut log = Log::new(path.clone())?;
        let keydir = log.load_index()?;
        println!("第二次加载索引，大小: {}", keydir.len());
        for (key, (pos, len)) in &keydir {
            println!("key: {:?}, pos: {}, len: {}", String::from_utf8_lossy(key), pos, len);
        }
        assert_eq!(2, keydir.len());

        Ok(())
    }

}

