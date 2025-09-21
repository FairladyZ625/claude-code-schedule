# Claude Code Schedule - systemd 服务配置指南

## 快速配置步骤

### 1. 准备工作
```bash
# 编译项目
cargo build --release

# 创建日志目录
sudo mkdir -p /var/log/claude-code-schedule
sudo chown $USER:$USER /var/log/claude-code-schedule
```

### 2. 配置服务参数

当前的服务配置已经设置为循环模式，执行时间为：**7:00, 12:00, 17:00, 22:00, 03:00**

如果您想修改参数，编辑服务文件中的 `ExecStart` 行：

```bash
sudo nano /etc/systemd/system/claude-code-schedule.service
```

### 3. 可用的配置选项

#### 基本配置（当前默认）
```ini
ExecStart=/home/root1/githubTools/claude-code-schedule/target/release/ccschedule --loop-mode --ping-mode --log-dir /var/log/claude-code-schedule --pid-file /var/run/claude-code-schedule.pid
```

#### 配置选项说明
- `--loop-mode`: 启用循环模式（7:00, 12:00, 17:00, 22:00, 03:00）
- `--ping-mode`: 使用ping模式（捕获Claude响应但轻量化）
- `--log-dir`: 日志存储目录
- `--pid-file`: PID文件位置
- `--message`: 自定义发送给Claude的消息

#### 其他配置示例

**使用完整Claude命令（非ping模式）：**
```ini
ExecStart=/home/root1/githubTools/claude-code-schedule/target/release/ccschedule --loop-mode --log-dir /var/log/claude-code-schedule --pid-file /var/run/claude-code-schedule.pid
```

**自定义消息：**
```ini
ExecStart=/home/root1/githubTools/claude-code-schedule/target/release/ccschedule --loop-mode --ping-mode --message "Check project status and provide updates" --log-dir /var/log/claude-code-schedule --pid-file /var/run/claude-code-schedule.pid
```

**不同的日志目录：**
```ini
ExecStart=/home/root1/githubTools/claude-code-schedule/target/release/ccschedule --loop-mode --ping-mode --log-dir /home/user/claude-logs --pid-file /var/run/claude-code-schedule.pid
```

### 4. 安装和启动服务

```bash
# 复制服务文件到系统目录
sudo cp claude-code-schedule.service /etc/systemd/system/

# 重新加载systemd配置
sudo systemctl daemon-reload

# 启用服务（开机自启）
sudo systemctl enable claude-code-schedule

# 启动服务
sudo systemctl start claude-code-schedule
```

### 5. 管理服务

```bash
# 查看服务状态
sudo systemctl status claude-code-schedule

# 查看实时日志
sudo journalctl -u claude-code-schedule -f

# 查看最近的日志
sudo journalctl -u claude-code-schedule --since "1 hour ago"

# 重启服务
sudo systemctl restart claude-code-schedule

# 停止服务
sudo systemctl stop claude-code-schedule

# 禁用服务
sudo systemctl disable claude-code-schedule
```

### 6. 监控和日志

#### 查看应用日志（JSON格式）
```bash
# 查看今天的日志
sudo tail -f /var/log/claude-code-schedule/$(date +%Y-%m-%d).log

# 查看最近的日志文件
ls -la /var/log/claude-code-schedule/

# 格式化查看JSON日志
sudo tail /var/log/claude-code-schedule/$(date +%Y-%m-%d).log | jq .
```

#### 查看系统日志
```bash
# 查看服务启动日志
sudo journalctl -u claude-code-schedule --since today

# 查看错误日志
sudo journalctl -u claude-code-schedule -p err

# 查看详细日志
sudo journalctl -u claude-code-schedule -o verbose
```

### 7. 故障排除

#### 常见问题

**服务启动失败：**
```bash
# 检查服务状态
sudo systemctl status claude-code-schedule

# 查看详细错误
sudo journalctl -u claude-code-schedule --no-pager

# 检查可执行文件权限
ls -la /home/root1/githubTools/claude-code-schedule/target/release/ccschedule
```

**权限问题：**
```bash
# 确保可执行文件有执行权限
chmod +x /home/root1/githubTools/claude-code-schedule/target/release/ccschedule

# 确保日志目录可写
sudo chown -R $USER:$USER /var/log/claude-code-schedule
```

**测试配置：**
```bash
# 手动测试命令
/home/root1/githubTools/claude-code-schedule/target/release/ccschedule --dry-run --loop-mode --ping-mode

# 测试短时间运行
timeout 30s /home/root1/githubTools/claude-code-schedule/target/release/ccschedule --loop-mode --ping-mode --log-dir /tmp/test-logs
```

### 8. 高级配置

#### 修改执行时间
如果您想要不同的执行时间，需要修改源代码中的 `get_loop_schedule()` 函数，然后重新编译：

```rust
fn get_loop_schedule() -> Vec<(u32, u32)> {
    // 自定义时间：例如每4小时执行一次
    vec![(6, 0), (10, 0), (14, 0), (18, 0), (22, 0), (2, 0)]
}
```

#### 环境变量配置
在服务文件中添加环境变量：
```ini
Environment=RUST_LOG=debug
Environment=CLAUDE_API_KEY=your_api_key
```

### 9. 安全建议

1. **使用专用用户**：
```bash
sudo useradd --system --shell /bin/false claude-schedule
sudo chown claude-schedule:claude-schedule /var/log/claude-code-schedule
```

2. **限制权限**：
修改服务文件中的 User 和 Group：
```ini
User=claude-schedule
Group=claude-schedule
```

3. **定期备份日志**：
设置logrotate配置自动轮转日志文件。
