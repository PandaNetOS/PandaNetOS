# 配置标准

## 配置格式

所有项目使用 **YAML** 作为主配置格式，文件扩展名 `.yaml`（禁止 `.yml`）。

## 配置加载优先级

从高到低：

1. **命令行参数**（最高优先级）
2. **环境变量**
3. **配置文件**
4. **代码默认值**（最低优先级）

## 环境变量命名

- 全大写 + 下划线
- 项目前缀：`SPDE_`、`PK_`、`PCDN_`
- 嵌套配置用双下划线分隔

示例：

```yaml
# config.yaml
agent:
  master: "http://localhost:5566"
  heartbeat_interval_secs: 10

global:
  max_concurrent: 4
```

对应环境变量：

```bash
SPDE_AGENT__MASTER="http://localhost:5566"
SPDE_AGENT__HEARTBEAT_INTERVAL_SECS=10
SPDE_GLOBAL__MAX_CONCURRENT=4
```

## 配置项规范

### 必填字段

- 必须有默认值（在代码中定义）
- 配置文件中缺失时使用默认值，不报错

### 数值范围

- 数值型配置应在文档中标注合法范围
- 超出范围时启动报错并退出，不静默截断

### 路径配置

- 相对路径相对于**工作目录**（二进制同级目录）
- 支持 `~` 展开为用户主目录
- 路径分隔符使用 `/`，跨平台自动转换

## 标准配置结构

### spde 节点配置

```yaml
agent:
  master: ""                    # 主控地址；可用 --master 覆盖
  node_id: null                 # 节点 UUID；留空使用本地 node-id.json
  heartbeat_interval_secs: 5    # 心跳间隔（秒）

global:
  work_dir: null                # 数据目录；null 使用默认
  max_concurrent: 4             # 最大并发下载任务数
  resume: true                  # 断点续传开关
  retry_times: 3                # 分片失败重试次数
  timeout: 1800                 # 单任务超时（秒）
  skip_tls_verify: false        # 跳过 TLS 证书校验
  connections_per_file: 8       # 单文件多连接数
  dry_run: false                # 试运行：不落盘

output:
  save_path: "./download"       # 保存目录

proxy:
  http_proxy: ""                # HTTP 代理地址
  https_proxy: ""               # HTTPS 代理地址

controller:
  url: ""                       # 主控地址；优先级低于 --master
  token: ""                     # Bearer Token；优先级低于 --token

direct_tasks:                   # 直接任务列表
  - name: "任务名称"
    enable: true
    url: "https://example.com/file.zip"
    filename: "file.zip"
```

### pk 主控配置

```yaml
listen: "0.0.0.0:5566"              # 监听地址
heartbeat_timeout_secs: 45          # 节点心跳超时（秒）
token: ""                           # API 鉴权 Token；空为不启用
spde_defaults:
  max_concurrent: 4
  connections_per_file: 8
  save_path: "./download"
```

## 配置文件位置

| 项目 | 配置文件路径 |
|------|-------------|
| spde | `{binary_dir}/spde-node/config/config.yaml` |
| pk | `{binary_dir}/pk-controlcenter/config.yaml` |

## 配置校验

启动时必须执行配置校验：

1. 必填字段存在且类型正确
2. 数值字段在合法范围内
3. 路径字段可创建（目录不存在时自动创建）
4. 校验失败时输出明确错误信息并以非零状态码退出

## 配置变更

- 运行时配置变更通过 WebSocket `config_changed` 消息通知
- 节点收到通知后重新拉取配置并热更新（无需重启）
- 不可热更新的配置项（如监听端口）需重启生效，应在文档中标注
