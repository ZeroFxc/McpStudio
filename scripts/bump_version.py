"""项目版本号统一更新工具
用法: python scripts/bump_version.py 0.1.3
"""

import re
import sys
from datetime import date
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
VERSION_RE = re.compile(r"^\d+\.\d+\.\d+$")
GITHUB_REPO = "ZeroFxc/McpStudio"


def update_simple(file_path: Path, pattern: str, template: str, new_version: str) -> bool:
    """简单正则替换"""
    content = file_path.read_text(encoding="utf-8")
    new_content = re.sub(pattern, template.format(new_version), content, count=1)
    if new_content == content:
        return False
    file_path.write_text(new_content, encoding="utf-8")
    return True


def update_changelog(file_path: Path, new_version: str) -> bool:
    """更新 CHANGELOG：Unreleased 链接 + 插入新版本标题 + 新版本链接"""
    content = file_path.read_text(encoding="utf-8")

    # 提取旧版本号
    m = re.search(r"\[Unreleased\]:.*?/compare/v(\d+\.\d+\.\d+)\.\.\.HEAD", content)
    if not m:
        return False
    old_version = m.group(1)

    # 更新 Unreleased 比较链接
    content = content.replace(
        f"/compare/v{old_version}...HEAD",
        f"/compare/v{new_version}...HEAD",
    )

    # 在 [Unreleased] 段落后插入新版本标题
    today = date.today().isoformat()
    content = re.sub(
        r"(## \[Unreleased\]\n\n)",
        f"\\1## [{new_version}] - {today}\n\n",
        content,
    )

    # 在链接区插入新版本链接
    content = re.sub(
        r"(\[Unreleased\]:.*?\n)",
        f"\\1[{new_version}]: https://github.com/{GITHUB_REPO}/releases/tag/v{new_version}\n",
        content,
    )

    file_path.write_text(content, encoding="utf-8")
    return True


def update_mcp_client(file_path: Path, new_version: str) -> bool:
    """更新 mcp_client.rs 中 MCP 客户端实现版本号"""
    content = file_path.read_text(encoding="utf-8")
    new_content = re.sub(
        r'Implementation::new\("McpStudio",\s*"\d+\.\d+\.\d+"\)',
        f'Implementation::new("McpStudio", "{new_version}")',
        content,
    )
    if new_content == content:
        return False
    file_path.write_text(new_content, encoding="utf-8")
    return True


def bump_version(new_version: str) -> None:
    if not VERSION_RE.match(new_version):
        print(f"错误: 版本号格式无效 '{new_version}'，应为 x.y.z")
        sys.exit(1)

    # 简单正则替换
    rules = [
        (ROOT / "package.json", r'"version":\s*"\d+\.\d+\.\d+"', '"version": "{}"'),
        (ROOT / "src-tauri" / "tauri.conf.json", r'"version":\s*"\d+\.\d+\.\d+"', '"version": "{}"'),
        (ROOT / "src-tauri" / "Cargo.toml", r'version\s*=\s*"\d+\.\d+\.\d+"', 'version = "{}"'),
        (ROOT / "src" / "components" / "SettingsPage.vue", r'<span class="about-value">\d+\.\d+\.\d+</span>', '<span class="about-value">{}</span>'),
    ]

    for file_path, pattern, template in rules:
        if not file_path.exists():
            print(f"警告: 文件不存在 {file_path.relative_to(ROOT)}，跳过")
            continue
        if update_simple(file_path, pattern, template, new_version):
            print(f"已更新: {file_path.relative_to(ROOT)}")
        else:
            print(f"警告: 未在 {file_path.relative_to(ROOT)} 中找到版本号，跳过")

    # CHANGELOG
    for changelog in [ROOT / "CHANGELOG.md", ROOT / "CHANGELOG-en.md"]:
        if not changelog.exists():
            print(f"警告: 文件不存在 {changelog.relative_to(ROOT)}，跳过")
            continue
        if update_changelog(changelog, new_version):
            print(f"已更新: {changelog.relative_to(ROOT)}")
        else:
            print(f"警告: 未在 {changelog.relative_to(ROOT)} 中找到 Unreleased 链接，跳过")

    # mcp_client.rs
    mcp_client = ROOT / "src-tauri" / "src" / "mcp_client.rs"
    if mcp_client.exists():
        if update_mcp_client(mcp_client, new_version):
            print(f"已更新: {mcp_client.relative_to(ROOT)}")
        else:
            print(f"警告: 未在 {mcp_client.relative_to(ROOT)} 中找到 MCP 客户端版本号，跳过")

    print(f"\n版本号已更新为 {new_version}")
    print("请运行 npm install 同步 package-lock.json")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("用法: python scripts/bump_version.py <版本号>")
        print("示例: python scripts/bump_version.py 0.1.3")
        sys.exit(1)

    bump_version(sys.argv[1])