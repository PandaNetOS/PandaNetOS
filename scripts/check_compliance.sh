#!/usr/bin/env bash
# =============================================================================
# PandaNetOS 生态合规检查脚本（v2.0）
# =============================================================================
# 所有项目 CI 必须调用本脚本，失败则阻断合并。
# 本脚本放在 PandaNetOS/scripts/check_compliance.sh，所有项目共用。
#
# 检查项（共10项）：
#   1. 标准库依赖检查（Rust 项目强制）
#   2. 目录布局检查（强制）
#   3. README 规范检查（强制，统一格式+统一顺序）
#   4. 代码格式检查（Rust 项目强制）
#   5. Clippy 检查（Rust 项目强制）
#   6. 单元测试（Rust 项目强制）
#   7. 敏感信息检查（强制）
#   8. 代码规范检查（Rust 项目强制）
#   9. Tag Guard 工作流检查（强制）
#  10. CI/CD 工作流完整性检查（强制）
#
# 用法：bash check_compliance.sh <项目根目录>
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
echo " PandaNetOS 生态合规检查 v2.0"
echo " 项目目录: $PROJECT_DIR"
echo " 检查时间: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
echo "========================================"

cd "$PROJECT_DIR"

# ---- 项目类型检测 ----
IS_RUST_PROJECT=0
if [ -f "Cargo.toml" ]; then
  IS_RUST_PROJECT=1
  echo "项目类型: Rust 项目"
else
  echo "项目类型: 非 Rust 项目"
fi

# =============================================================================
# [1/10] 标准库依赖检查（Rust 项目强制）
# =============================================================================
section "1/10" "标准库依赖检查"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过"
else
  if grep -qE 'pandanetos\s*=\s*\{[^}]*path\s*=\s*"\.\./PandaNetOS/crates/pandanetos"' Cargo.toml; then
    pass "使用 path 依赖 pandanetos"
  else
    fail "未使用 path 依赖 pandanetos"
  fi

  if grep -qE 'pandanetos\s*=\s*\{[^}]*git\s*=' Cargo.toml; then
    fail "使用了 git 依赖 pandanetos（本地开发必须用 path 依赖）"
  else
    pass "未使用 git 依赖"
  fi

  if grep -rqE 'const\s+API_PREFIX|const\s+AGENT_REGISTER|const\s+AGENT_WS' src/ 2>/dev/null; then
    fail "存在私有 API 路径常量（必须复用 pandanetos::protocol::paths）"
  else
    pass "未发现私有 API 路径常量"
  fi
fi

# =============================================================================
# [2/10] 目录布局检查（强制）
# =============================================================================
section "2/10" "目录布局检查"

if [ -d "../PandaNetOS/crates/pandanetos" ]; then
  pass "PandaNetOS 标准库存在于同级目录"
else
  fail "未找到 ../PandaNetOS/crates/pandanetos"
fi

if [ "$IS_RUST_PROJECT" -eq 1 ]; then
  if [ -d "src" ]; then
    pass "src/ 目录存在"
  else
    fail "src/ 目录不存在"
  fi
else
  skip "非 Rust 项目，跳过 src/ 检查"
fi

# =============================================================================
# [3/10] README 规范检查（强制，统一格式+统一顺序）
# =============================================================================
section "3/10" "README 规范检查（统一格式+统一顺序）"

if [ ! -f "README.md" ]; then
  fail "README.md 不存在"
else
  README_ERRORS=0

  # ---- 强制章节（FAIL）----
  if grep -qE '^# ' README.md; then
    pass "包含项目标题"
  else
    fail "缺少项目标题（必须以 # 开头）"
    README_ERRORS=$((README_ERRORS + 1))
  fi

  if grep -qE '^## .*功能特性|^## .*Features|^## .*特性|^## .*功能' README.md; then
    pass "包含「功能特性」章节"
  else
    fail "缺少「功能特性」章节"
    README_ERRORS=$((README_ERRORS + 1))
  fi

  if grep -q "标准库路径约定" README.md; then
    pass "包含「标准库路径约定」章节"
  else
    fail "缺少「标准库路径约定」章节"
    README_ERRORS=$((README_ERRORS + 1))
  fi

  if grep -q "PandaNetOS" README.md; then
    pass "引用了 PandaNetOS 生态"
  else
    fail "未引用 PandaNetOS 生态"
    README_ERRORS=$((README_ERRORS + 1))
  fi

  if grep -q 'path = "../PandaNetOS/crates/pandanetos"' README.md; then
    pass "写明了 path 依赖写法"
  else
    fail "未写明 path 依赖写法"
    README_ERRORS=$((README_ERRORS + 1))
  fi

  if grep -qE '^## .*快速开始|^## .*Quick Start|^## .*快速使用|^## .*使用方法|^## .*Getting Started' README.md; then
    pass "包含「快速开始」章节"
  else
    fail "缺少「快速开始」章节"
    README_ERRORS=$((README_ERRORS + 1))
  fi

  if grep -qE '^## .*开发指南|^## .*Development|^## .*开发|^## .*贡献|^## .*Contributing' README.md; then
    pass "包含「开发指南/贡献」章节"
  else
    fail "缺少「开发指南/贡献」章节"
    README_ERRORS=$((README_ERRORS + 1))
  fi

  if grep -qiE '^## .*许可|^## .*License|^## .*版权' README.md; then
    pass "包含「许可证」章节"
    if grep -qiE 'MIT' README.md; then
      pass "许可证为 MIT（统一标准）"
    else
      warn "建议统一使用 MIT 许可证"
    fi
  else
    fail "缺少「许可证」章节（统一 MIT）"
    README_ERRORS=$((README_ERRORS + 1))
  fi

  # ---- 建议子章节（WARN）----
  if grep -qE '^### .*环境要求|^### .*Requirements|^### .*前置条件|^### .*Prerequisites' README.md; then
    pass "快速开始包含「环境要求」"
  else
    warn "快速开始建议包含「环境要求」子章节"
  fi

  if grep -qE '^### .*安装|^### .*Install|^### .*构建|^### .*Build' README.md; then
    pass "快速开始包含「安装/构建」"
  else
    warn "快速开始建议包含「安装/构建」子章节"
  fi

  if grep -qE '^### .*使用示例|^### .*Usage|^### .*示例|^### .*Example' README.md; then
    pass "快速开始包含「使用示例」"
  else
    warn "快速开始建议包含「使用示例」子章节"
  fi

  if grep -qE '^### .*构建|^### .*Build|cargo build|docker build' README.md; then
    pass "开发指南包含「构建」说明"
  else
    warn "开发指南建议包含「构建」说明"
  fi

  if grep -qE '^### .*测试|^### .*Test|cargo test' README.md; then
    pass "开发指南包含「测试」说明"
  else
    warn "开发指南建议包含「测试」说明"
  fi

  if grep -qE '合规检查|check_compliance|compliance' README.md; then
    pass "开发指南包含「合规检查」说明"
  else
    warn "开发指南建议包含「合规检查」说明"
  fi

  # ---- 建议章节（WARN）----
  if grep -qE '^## .*配置|^## .*Config|^## .*参数' README.md; then
    pass "包含「配置说明」章节"
  else
    warn "建议包含「配置说明」章节"
  fi

  if grep -qE '^## .*项目结构|^## .*Structure|^## .*目录结构' README.md; then
    pass "包含「项目结构」章节"
  else
    warn "建议包含「项目结构」章节"
  fi

  if grep -qE '^## .*变更日志|^## .*Changelog|^## .*更新日志|CHANGELOG' README.md; then
    pass "包含「变更日志」章节"
  else
    warn "建议包含「变更日志」章节或链接到 CHANGELOG.md"
  fi

  if grep -qE '\[!\[.*\]\(.*\)\]|img.shields.io' README.md; then
    pass "包含徽章"
  else
    warn "建议包含许可证/构建状态等徽章"
  fi

  if grep -qE '^> |^[^#].{10,}' README.md; then
    pass "包含项目简介"
  else
    warn "建议包含一句话项目简介"
  fi

  # ---- README 章节顺序强制检查（FAIL）----
  SECTIONS=$(grep -nE '^## ' README.md 2>/dev/null || true)
  if [ -n "$SECTIONS" ]; then
    STANDARD_ORDER=("功能特性" "标准库路径约定" "快速开始" "配置说明" "项目结构" "开发指南" "贡献指南" "变更日志" "许可证")
    PREV_LINE=0
    PREV_NAME=""
    ORDER_ERROR=0
    
    for STD_NAME in "${STANDARD_ORDER[@]}"; do
      LINE_NUM=$(echo "$SECTIONS" | grep -E "## .*${STD_NAME}" | head -1 | cut -d: -f1)
      if [ -n "$LINE_NUM" ]; then
        if [ "$LINE_NUM" -lt "$PREV_LINE" ]; then
          fail "README 章节顺序错误：「${STD_NAME}」在「${PREV_NAME}」之前"
          ORDER_ERROR=$((ORDER_ERROR + 1))
        else
          PREV_LINE=$LINE_NUM
          PREV_NAME=$STD_NAME
        fi
      fi
    done
    
    if [ "$ORDER_ERROR" -eq 0 ]; then
      pass "README 章节顺序符合标准"
    fi
  else
    warn "未检测到 ## 章节，无法校验顺序"
  fi

  if [ "$README_ERRORS" -eq 0 ]; then
    pass "README 所有强制检查项通过"
  fi
fi

# =============================================================================
# [4/10] 代码格式检查（Rust 项目强制）
# =============================================================================
section "4/10" "代码格式检查"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过"
elif command -v cargo &>/dev/null; then
  if cargo fmt --all -- --check 2>&1; then
    pass "cargo fmt 检查通过"
  else
    fail "cargo fmt 检查未通过"
  fi
else
  warn "cargo 未安装，跳过"
fi

# =============================================================================
# [5/10] Clippy 检查（Rust 项目强制）
# =============================================================================
section "5/10" "Clippy 检查"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过"
elif command -v cargo &>/dev/null; then
  if cargo clippy --all-targets -- -D warnings 2>&1 | tail -5; then
    pass "cargo clippy 检查通过"
  else
    fail "cargo clippy 检查未通过"
  fi
else
  warn "cargo 未安装，跳过"
fi

# =============================================================================
# [6/10] 单元测试（Rust 项目强制）
# =============================================================================
section "6/10" "单元测试"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过"
elif command -v cargo &>/dev/null; then
  if cargo test 2>&1 | tail -10; then
    pass "cargo test 全部通过"
  else
    fail "cargo test 存在失败"
  fi
else
  warn "cargo 未安装，跳过"
fi

# =============================================================================
# [7/10] 敏感信息检查（强制）
# =============================================================================
section "7/10" "敏感信息检查"

SENSITIVE_FOUND=0

if grep -rn --include="*.rs" --include="*.yaml" --include="*.yml" --include="*.toml" --include="*.sh" \
  -E '(token|password|api_key|secret)\s*=\s*"[a-zA-Z0-9_\-]{8,}"' \
  src/ Cargo.toml *.yaml *.yml *.sh 2>/dev/null | \
  grep -vE '""|null|placeholder|example|your_|<.*>|panda-allow' | \
  grep -v 'target/'; then
  fail "发现硬编码敏感信息"
  SENSITIVE_FOUND=1
fi

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
# [8/10] 代码规范检查（Rust 项目强制）
# =============================================================================
section "8/10" "代码规范检查"

if [ "$IS_RUST_PROJECT" -eq 0 ]; then
  skip "非 Rust 项目，跳过"
else
  FORBIDDEN_FOUND=0
  FORBIDDEN_LINES=$(grep -rn --include="*.rs" -E '\b(println!|dbg!|todo!|unimplemented!)\(' src/ 2>/dev/null | \
    grep -v 'panda-allow:' || true)
  if [ -n "$FORBIDDEN_LINES" ]; then
    echo "$FORBIDDEN_LINES"
    fail "src/ 中存在 println!/dbg!/todo!/unimplemented!"
    FORBIDDEN_FOUND=1
  fi

  UNWRAP_COUNT=$(grep -rn --include="*.rs" -E '\.unwrap\(\)' src/ 2>/dev/null | \
    grep -vc 'SAFETY\|安全\|//.*unwrap\|panda-allow' || true)
  if [ "$UNWRAP_COUNT" -gt 0 ]; then
    warn "src/ 中存在 $UNWRAP_COUNT 处 unwrap()（建议用 ? 或 expect）"
  else
    pass "未发现无注释的 unwrap()"
  fi

  if [ "$FORBIDDEN_FOUND" -eq 0 ]; then
    pass "未发现禁用宏"
  fi
fi

# =============================================================================
# [9/10] Tag Guard 工作流检查（强制）
# =============================================================================
section "9/10" "Tag Guard 工作流检查"

TAG_GUARD_FILE=".github/workflows/tag-guard.yml"
if [ ! -f "$TAG_GUARD_FILE" ]; then
  fail "缺少 Tag Guard 工作流: $TAG_GUARD_FILE"
  fail "必须添加 Tag Guard，参考 PandaNetOS/actions/tag-guard"
else
  pass "Tag Guard 工作流文件存在"

  if grep -q 'PandaNetOS/PandaNetOS/actions/tag-guard@' "$TAG_GUARD_FILE"; then
    pass "引用了 PandaNetOS 标准库的 Tag Guard action"
  else
    fail "必须引用 PandaNetOS/PandaNetOS/actions/tag-guard@main"
  fi

  if grep -q 'delete-on-failure:.*true' "$TAG_GUARD_FILE"; then
    pass "已启用 delete-on-failure"
  else
    fail "必须启用 delete-on-failure: true"
  fi

  if grep -q 'check-ci-status:.*true' "$TAG_GUARD_FILE"; then
    pass "已启用 check-ci-status"
  else
    fail "必须启用 check-ci-status: true"
  fi

  if grep -q 'github-token:' "$TAG_GUARD_FILE"; then
    pass "已配置 github-token"
  else
    fail "必须配置 github-token"
  fi
fi

# =============================================================================
# [10/10] CI/CD 工作流完整性检查（强制）
# =============================================================================
section "10/10" "CI/CD 工作流完整性检查"

if [ -f ".github/workflows/cargo-test.yml" ] || grep -rq 'cargo test' .github/workflows/ 2>/dev/null; then
  pass "存在 Cargo Test 工作流"
else
  warn "建议添加 Cargo Test 工作流"
fi

if [ -f ".github/workflows/cargo-format.yml" ] || grep -rq 'cargo fmt' .github/workflows/ 2>/dev/null; then
  pass "存在 Cargo Format 工作流"
else
  warn "建议添加 Cargo Format 工作流"
fi

if [ -f ".github/workflows/cargo-clippy.yml" ] || grep -rq 'cargo clippy' .github/workflows/ 2>/dev/null; then
  pass "存在 Cargo Clippy 工作流"
else
  warn "建议添加 Cargo Clippy 工作流"
fi

if [ -f ".github/workflows/compliance.yml" ] || grep -rq 'check_compliance' .github/workflows/ 2>/dev/null; then
  pass "存在 Compliance 工作流"
else
  fail "必须添加 Compliance 工作流（调用 PandaNetOS/scripts/check_compliance.sh）"
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
