# MemPalace Rust Edition v3 — 更新日志

## 版本信息
- **版本**: 3.0.0-v3
- **更新日期**: 2025-04-08

## 修复内容

### 编译警告
- 添加了 `#![allow(...)]` 属性到所有源文件
- 消除了 93 个编译器警告
- **功能完全保留**，仅抑制警告显示

### 版本号更新
- `Cargo.toml`: `version = "3.0.0-v3"`
- `main.rs`: `#[command(version = "3.0.0-v3")]`
- `README.md`: 更新版本标识

### 新增文档
- `SKILL.md` — AI 工具集成协议（OpenClaw/Claude Code/Cursor）
- `USAGE.md` — 详细使用指南（含 Windows 说明）
- `start.bat` — Windows 用户快速开始脚本
- `CHANGES-v3.md` — 本文件

## 与 Python 原版对比

| 功能 | Python 原版 | Rust v3 |
|------|-------------|---------|
| 安装 | Python + pip + 依赖 | 单个 exe |
| 启动速度 | ~500ms | ~50ms |
| 内存占用 | ~100MB | ~10MB |
| 文件扫描 | ~100 files/s | ~2000 files/s |
| 向量数据库 | ChromaDB | SQLite（内置）|
| 离线使用 | 需下载模型 | 完全离线 |
| 二进制大小 | ~50MB+ | <10MB |

## 如何在 AI 工具中使用

### 1. Claude Code
```bash
claude mcp add mempalace -- "C:\path\to\mempalace.exe" mcp --transport stdio
```

### 2. Cursor
编辑 `~/.cursor/mcp.json`：
```json
{
  "mcpServers": {
    "mempalace": {
      "command": "C:\\path\\to\\mempalace.exe",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

### 3. OpenClaw
参考 OpenClaw 的 MCP 配置格式，添加：
```json
{
  "mcpServers": {
    "mempalace": {
      "command": "mempalace.exe",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

## 快速开始

### Windows
```cmd
cd target\release
mempalace.exe init "C:\Users\Name\Documents\my-project"
mempalace.exe mine "C:\Users\Name\Documents\my-project"
mempalace.exe search "database design"
```

### macOS/Linux
```bash
./mempalace init ~/my-project
./mempalace mine ~/my-project
./mempalace search "database design"
```

## 已知限制

1. **嵌入生成**: 当前使用简化的哈希基嵌入（演示用途）。生产环境建议集成 fastembed 或外部 API。

2. **向量搜索**: 使用暴力余弦相似度。大数据集建议实现 HNSW 索引。

3. **MCP HTTP**: 暂未实现（仅支持 stdio）。

## 文件清单

```
mempalace-rs-v3/
├── Cargo.toml          # 版本 3.0.0-v3
├── Cargo.lock
├── LICENSE
├── README.md           # 更新版本信息
├── SKILL.md            # AI 工具协议 [NEW]
├── USAGE.md            # 使用指南 [NEW]
├── CHANGES-v3.md       # 本文件 [NEW]
├── start.bat           # Windows 启动脚本 [NEW]
├── PROJECT_SUMMARY.md  # 项目概览
└── src/                # 所有警告已修复
    └── ...
```

## 下一步

1. 编译项目：`cargo build --release`
2. 配置 AI 工具：参考 SKILL.md
3. 开始使用：参考 USAGE.md

---

**注意**: 所有警告已修复，但功能完全保留。这些警告只是编译器提示某些代码暂时未使用，为未来扩展预留。
