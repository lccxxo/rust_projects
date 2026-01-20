use fs4::FileExt;
use std::{
    collections::BTreeMap,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    ops::Bound,
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

    fn flush(&mut self)->Result<()>{
        Ok(self.log.file.sync_all()?)
    }


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

    fn load_index(&self) -> Result<KyeDir> {
        let keydir = KyeDir::new();

        Ok(keydir)
    }

    // 根据 value 的位置和长度获取 value 的值
    fn read_value(&mut self, value_pos: u64, value_len: u32) -> Result<Vec<u8>> {
        let mut value = vec![0; value_len as usize];
        Ok(value)
    }

    // +-------------+-------------+----------------+----------------+
    // | key len(4)    val len(4)     key(varint)       val(varint)  |
    // +-------------+-------------+----------------+----------------+
    fn write_entry(&mut self, key: &[u8], value: Option<&[u8]>) -> Result<(u64, u32)> {
        Ok((0, 0))
    }
}
