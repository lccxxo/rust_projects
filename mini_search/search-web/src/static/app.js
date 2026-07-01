const form = document.getElementById("search-form");
const queryInput = document.getElementById("query-input");
const statusBar = document.getElementById("status-bar");
const resultsDiv = document.getElementById("results");
const crawlForm = document.getElementById("crawl-form");
const crawlMsg = document.getElementById("crawl-msg");
const crawlProgress = document.getElementById("crawl-progress");
const crawlFill = crawlProgress.querySelector(".crawl-fill");
const crawledPages = document.getElementById("crawled-pages");

let currentQuery = "";
let currentPage = 1;
let pollTimer = null;

// ── Search ─────────────────────────────────────────────────

form.addEventListener("submit", (e) => {
    e.preventDefault();
    const q = queryInput.value.trim();
    if (!q) return;
    currentQuery = q;
    currentPage = 1;
    search(q, 1);
});

async function search(q, page) {
    resultsDiv.innerHTML = '<div class="empty-state">搜索中...</div>';
    statusBar.textContent = "";

    try {
        const resp = await fetch(`/api/search?q=${encodeURIComponent(q)}&page=${page}`);
        const data = await resp.json();

        if (data.results.length === 0) {
            resultsDiv.innerHTML = `<div class="empty-state">没有找到与 "<strong>${escapeHtml(q)}</strong>" 相关的结果。</div>`;
            statusBar.textContent = "(索引中暂无匹配文档)";
            return;
        }

        statusBar.textContent = `找到约 ${data.total} 条结果`;

        let html = "";
        for (const r of data.results) {
            html += `
                <div class="result-item">
                    <a class="title" href="${escapeAttr(r.url)}" target="_blank">${escapeHtml(r.title) || "(无标题)"}</a>
                    <div class="url">${escapeHtml(r.url)}</div>
                    <div class="snippet">${r.snippet}</div>
                    <div class="score">相关性: ${r.score.toFixed(2)}</div>
                </div>`;
        }

        if (data.total > data.page_size) {
            const totalPages = Math.ceil(data.total / data.page_size);
            html += '<div class="pagination">';
            html += `<button ${page <= 1 ? "disabled" : ""} data-page="${page - 1}">上一页</button>`;
            for (let p = 1; p <= totalPages && p <= 10; p++) {
                html += `<button class="${p === page ? "active" : ""}" data-page="${p}">${p}</button>`;
            }
            html += `<button ${page >= totalPages ? "disabled" : ""} data-page="${page + 1}">下一页</button>`;
            html += "</div>";
        }

        resultsDiv.innerHTML = html;

        resultsDiv.querySelectorAll(".pagination button[data-page]").forEach((btn) => {
            btn.addEventListener("click", () => {
                const p = parseInt(btn.dataset.page);
                currentPage = p;
                search(currentQuery, p);
                window.scrollTo(0, 0);
            });
        });
    } catch (err) {
        resultsDiv.innerHTML = `<div class="empty-state">搜索失败: ${escapeHtml(err.message)}</div>`;
    }
}

// ── Crawl with progress polling ────────────────────────────

crawlForm.addEventListener("submit", async (e) => {
    e.preventDefault();
    const url = document.getElementById("crawl-url").value.trim();
    const depth = parseInt(document.getElementById("crawl-depth").value) || 1;

    if (!url) return;

    crawlProgress.classList.remove("hidden");
    crawlMsg.textContent = "正在启动爬虫...";
    crawlMsg.className = "";
    crawlFill.style.width = "0%";
    crawlFill.classList.add("running");

    try {
        const resp = await fetch("/api/crawl", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ url, max_depth: depth }),
        });
        await resp.json();
        document.getElementById("crawl-url").value = "";

        // Start polling for progress
        startPolling(url);
    } catch (err) {
        crawlMsg.textContent = `请求失败: ${err.message}`;
        crawlMsg.className = "error";
        crawlFill.classList.remove("running");
    }
});

function startPolling(url) {
    if (pollTimer) clearInterval(pollTimer);

    pollTimer = setInterval(async () => {
        try {
            const resp = await fetch("/api/status");
            const data = await resp.json();
            const cs = data.crawl;

            if (!cs.running) {
                clearInterval(pollTimer);
                pollTimer = null;
                crawlFill.classList.remove("running");
                crawlFill.style.width = "100%";

                if (cs.error) {
                    crawlMsg.textContent = `爬取失败: ${cs.error}`;
                    crawlMsg.className = "error";
                } else if (cs.pages_crawled === 0) {
                    crawlMsg.textContent = "未爬取到任何页面。目标网站可能无法访问或不是 HTML 页面。";
                    crawlMsg.className = "error";
                } else {
                    crawlMsg.textContent = `爬取完成: ${cs.pages_crawled} 个页面, ${cs.pages_indexed} 个已索引。索引现有 ${data.doc_count} 个文档。`;
                    crawlMsg.className = "";

                    // Show crawled pages list
                    if (cs.pages && cs.pages.length > 0) {
                        let listHtml = '<h3>已爬取页面:</h3><ul>';
                        for (const p of cs.pages) {
                            listHtml += `<li><a href="${escapeAttr(p.url)}" target="_blank">${escapeHtml(p.title) || p.url}</a></li>`;
                        }
                        listHtml += '</ul>';
                        crawledPages.innerHTML = listHtml;
                        crawledPages.classList.remove("hidden");
                    }

                    // Auto-search using hostname (URL field is now searchable)
                    let host = "";
                    if (cs.url) {
                        try { host = new URL(cs.url).hostname; } catch (_) {}
                    }
                    if (host) {
                        currentQuery = host;
                        currentPage = 1;
                        queryInput.value = currentQuery;
                        search(currentQuery, 1);
                    }

                    setTimeout(() => {
                        crawlFill.style.width = "0%";
                        crawlProgress.classList.add("hidden");
                    }, 5000);
                }
            } else {
                crawlFill.style.width = "70%";
                crawlMsg.textContent = `爬取中: ${cs.url} ... 已爬取 ${cs.pages_crawled} 页`;
                crawlMsg.className = "";
            }
        } catch (err) {
            // ignore polling errors
        }
    }, 1500);
}

// ── Helpers ────────────────────────────────────────────────

function escapeHtml(str) {
    const div = document.createElement("div");
    div.textContent = str;
    return div.innerHTML;
}

function escapeAttr(str) {
    return str.replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
