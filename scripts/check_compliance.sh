#!/usr/bin/env bash
# =============================================================================
# PandaNetOS 生态合规检查脚本
# =============================================================================
# 所有项目 CI 必须调用本脚本，失败则阻断合并。
# 本脚本放在 PandaNetOS/scripts/check_compliance.sh，所有项目共用。
#
# 支持项目类型：
#   - Rust 项目（有 Cargo.toml）：全部8项检查
#   - 非 Rust 项目（Docker/Shell等）：跳过 Rust 相关检查，仅检查通用项
#
# 用法：
#   bash check_compliance.sh <项目根目录>
#   bash check_compliance.sh .          # 检查当前目录
#
# 白名单注释：
#   // panda-allow: cli-output    允许该行使用 println!（CLI 输出场景）
#
# 退出码：0=全部通过，1=存在问题
# =============================================================================

set -euo pipefail

PROJECT_DIR="${1:-.}"
ERRORS=0
WARNINGS=0

# ---- 颜色输出 ----
if [ -t 1 ]; then
  RED='\033[31m'; GREEN='\033[32m'; YELLOW='\033[33m'; CYAN='\033[36m'; RESET='\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; CYAN=''; RESET=''
fi

fail()    { echo -e "${RED}  ❌ FAIL: $1${RESET}"; ERRORS=$((ERRORS + 1)); }
pass()    { echo -e "${GREEN}  ✅ PASS: $1${RESET}"; }
warn()    { echo -e "${YELLOW}  ⚠️  WARN: $1${RESET}"; WARNINGS=$((WARNINGS + 1)); }
skip()    { echo -e "${CYAN}  ⏭️  SKIP: $1${RESET}"; }
section() { echo -e "\n${CYAN}[$1] $2${RESET}"; }

echo "========================================"
echo " PandaNetOS 生态合规检查"
echo " 项目目录: $PROJECT_DIR"
echo " 检查时间: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
echo "========================================"

# 切换到项目目录
cd "$PROJECT_DIR"

# ---- 项目类型检测 ----
IS_RUST_PROJECT=0
if [ -f "Cargo.toml" ]; then
  IS_RUST_PROJECT=1
  echo "项目类型: Rust 项目（检测到 Cargo.toml）"
else
  echo "项目类型: 非 Rust 项目（未检测到 Cargo.toml，跳过 Rust 相关检查）"
fi

# =============================================================================
# [1/8] 标准库依赖检查（Rust 项目强制）
# =============================================================================
section "1/8" "标准库依赖检查"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过标准库依赖检查"
else
  # 必须使用 path 依赖
  if grep -qE 'pandanetos\s*=\s*\{[^}]*path\s*=\s*"\.\./PandaNetOS/crates/pandanetos"' Cargo.toml; then
    pass "Cargo.toml 使用 path 依赖 pandanetos"
  else
    fail "Cargo.toml 未使用 path 依赖 pandanetos（必须: path = \"../PandaNetOS/crates/pandanetos\"）"
  fi

  # 禁止本地开发使用 git 依赖
  if grep -qE 'pandanetos\s*=\s*\{[^}]*git\s*=' Cargo.toml; then
    fail "Cargo.toml 使用了 git 依赖 pandanetos（本地开发必须用 path 依赖，git 依赖仅限 CI 发布构建）"
  else
    pass "未使用 git 依赖（符合本地开发要求）"
  fi

  # 禁止维护私有协议常量
  if grep -rqE 'const\s+API_PREFIX|const\s+AGENT_REGISTER|const\s+AGENT_WS' src/ 2>/dev/null; then
    fail "项目中存在私有 API 路径常量（必须复用 pandanetos::protocol::paths）"
  else
    pass "未发现私有 API 路径常量"
  fi
fi

# =============================================================================
# [2/8] 目录布局检查（强制）
# =============================================================================
section "2/8" "目录布局检查"

if [ -d "../PandaNetOS/crates/pandanetos" ]; then
  pass "PandaNetOS 标准库仓库存在于同级目录"
else
  fail "未找到 ../PandaNetOS/crates/pandanetos（标准库仓库必须与本项目同级）"
fi

# Rust 项目检查 src/ 目录
if [ "$IS_RUST_PROJECT" -eq 1 ]; then
  if [ -d "src" ]; then
    pass "src/ 目录存在"
  else
    fail "src/ 目录不存在"
  fi
else
  skip "非 Rust 项目，跳过 src/ 目录检查"
fi

# =============================================================================
# [3/8] README 规范检查（所有项目强制）
# =============================================================================
section "3/8" "README 规范检查"

if [ ! -f "README.md" ]; then
  fail "README.md 不存在"
else
  # 必须包含标准库路径约定章节
  if grep -q "标准库路径约定" README.md; then
    pass "README 包含「标准库路径约定」章节"
  else
    fail "README 缺少「标准库路径约定」章节（见 PandaNetOS docs/standards/project-structure.md）"
  fi

  # 必须引用 PandaNetOS 生态
  if grep -q "PandaNetOS" README.md; then
    pass "README 引用了 PandaNetOS 生态"
  else
    fail "README 未引用 PandaNetOS 生态"
  fi

  # 必须写明 path 依赖写法
  if grep -q 'path = "../PandaNetOS/crates/pandanetos"' README.md; then
    pass "README 写明了 path 依赖写法"
  else
    fail "README 未写明 path 依赖写法"
  fi

  # 必须包含快速开始
  if grep -qE '^## .*快速开始|^## .*Quick Start|^## .*快速使用|^## .*使用方法' README.md; then
    pass "README 包含「快速开始」章节"
  else
    warn "README 建议包含「快速开始」章节"
  fi

  # 必须包含许可证
  if grep -qiE '^## .*许可|^## .*License|MIT|Apache' README.md; then
    pass "README 包含许可证信息"
  else
    warn "README 建议包含许可证信息（统一 MIT）"
  fi
fi

# =============================================================================
# [4/8] 代码格式检查（Rust 项目强制）
# =============================================================================
section "4/8" "代码格式检查"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过 fmt 检查"
elif command -v cargo &>/dev/null; then
  if cargo fmt --all -- --check 2>&1; then
    pass "cargo fmt 检查通过"
  else
    fail "cargo fmt 检查未通过（执行: cargo fmt --all）"
  fi
else
  warn "cargo 未安装，跳过 fmt 检查"
fi

# =============================================================================
# [5/8] Clippy 检查（Rust 项目强制）
# =============================================================================
section "5/8" "Clippy 检查"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过 clippy 检查"
elif command -v cargo &>/dev/null; then
  if cargo clippy --all-targets -- -D warnings 2>&1 | tail -5; then
    pass "cargo clippy 检查通过（警告视为错误）"
  else
    fail "cargo clippy 检查未通过（存在 warning，CI 中 -D warnings 视为错误）"
  fi
else
  warn "cargo 未安装，跳过 clippy 检查"
fi

# =============================================================================
# [6/8] 单元测试（Rust 项目强制）
# =============================================================================
section "6/8" "单元测试"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过单元测试"
elif command -v cargo &>/dev/null; then
  if cargo test 2>&1 | tail -10; then
    pass "cargo test 全部通过"
  else
    fail "cargo test 存在失败"
  fi
else
  warn "cargo 未安装，跳过测试"
fi

# =============================================================================
# [7/8] 敏感信息检查（所有项目强制）
# =============================================================================
section "7/8" "敏感信息检查"

SENSITIVE_FOUND=0

# 检查硬编码 token / password / api_key
if grep -rn --include="*.rs" --include="*.yaml" --include="*.yml" --include="*.toml" --include="*.sh" \
  -E '(token|password|api_key|secret)\s*=\s*"[a-zA-Z0-9_\-]{8,}"' \
  src/ Cargo.toml *.yaml *.yml *.sh 2>/dev/null | \
  grep -vE '""|null|placeholder|example|your_|<.*>|panda-allow' | \
  grep -v 'target/'; then
  fail "发现硬编码敏感信息（token/password/api_key/secret）"
  SENSITIVE_FOUND=1
fi

# 检查私钥
if grep -rn --include="*.rs" --include="*.pem" --include="*.key" \
  -E 'BEGIN (RSA |EC |OPENSSH |DSA )?PRIVATE KEY' \
  . 2>/dev/null | grep -v 'target/'; then
  fail "发现私钥文件或内容"
  SENSITIVE_FOUND=1
fi

if [ "$SENSITIVE_FOUND" -eq 0 ]; then
  pass "未发现硬编码敏感信息"
fi

# =============================================================================
# [8/8] 代码规范检查（Rust 项目强制）
# =============================================================================
section "8/8" "代码规范检查"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过代码规范检查"
else
  # 禁止 println!/dbg!/todo! 在 src 中（支持白名单注释 // panda-allow: cli-output）
  FORBIDDEN_FOUND=0
  FORBIDDEN_LINES=$(grep -rn --include="*.rs" -E '\b(println!|dbg!|todo!|unimplemented!)\(' src/ 2>/dev/null | \
    grep -v 'panda-allow:' || true)
  if [ -n "$FORBIDDEN_LINES" ]; then
    echo "$FORBIDDEN_LINES"
    fail "src/ 中存在 println!/dbg!/todo!/unimplemented!（使用 tracing 替代日志；CLI 输出可加 // panda-allow: cli-output）"
    FORBIDDEN_FOUND=1
  fi

  # 禁止 unwrap() 无安全注释
  UNWRAP_COUNT=$(grep -rn --include="*.rs" -E '\.unwrap\(\)' src/ 2>/dev/null | \
    grep -vc 'SAFETY\|安全\|//.*unwrap\|panda-allow' || true)
  if [ "$UNWRAP_COUNT" -gt 0 ]; then
    warn "src/ 中存在 $UNWRAP_COUNT 处 unwrap()（建议用 ? 或 expect 并说明安全理由）"
  else
    pass "未发现无注释的 unwrap()"
  fi

  if [ "$FORBIDDEN_FOUND" -eq 0 ]; then
    pass "未发现禁用宏（println!/dbg!/todo!）"
  fi
fi

# =============================================================================
# 汇总
# =============================================================================
echo ""
echo "========================================"
echo " 检查结果汇总"
echo "========================================"

if [ "$ERRORS" -eq 0 ] && [ "$WARNINGS" -eq 0 ]; then
  echo -e "${GREEN}✅ 全部通过，无警告${RESET}"
  exit 0
elif [ "$ERRORS" -eq 0 ]; then
  echo -e "${YELLOW}⚠️  全部通过，但有 $WARNINGS 个警告（建议修复）${RESET}"
  exit 0
else
  echo -e "${RED}❌ 发现 $ERRORS 个必须修复的问题${RESET}"
  if [ "$WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  另有 $WARNINGS 个警告${RESET}"
  fi
  echo -e "${RED}必须修复后才能合并到 main${RESET}"
  exit 1
fi
