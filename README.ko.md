# herdr-obsidian

[English](README.md) | [**한국어**](README.ko.md)

지원 언어: 영어 · 한국어 · 러시아어 · 프랑스어 · 스페인어 · 독일어 · 일본어

herdr pane 하나에서 [Obsidian](https://obsidian.md) 볼트를 보고 편집합니다. 노트는 일반 Markdown 파일이라 어떤 폴더든 쓸 수 있고, Google Drive 데스크톱 앱으로 동기화된 폴더도 됩니다.

## 대시보드 디자인

```text
┌ plan.md ● ──────────────────────────┐┌ 볼트 목록 (↑↓ Enter 전환) ─────────┐
│# Plan                               ││● C:\work\obsidian\herdr_test       │
│                                     ││  C:\work\obsidian\test             │
│- [ ] README 쓰기                    │└────────────────────────────────────┘
│- [x] 자동 저장                      │┌ C:\work\obsidian\herdr_test ───────┐
│                                     ││▾ Projects                          │
│> 노트는 일반 Markdown 입니다        ││    plan.md                         │
│                                     ││▸ Archive                           │
│                                     ││  welcome.md                        │
└─────────────────────────────────────┘└────────────────────────────────────┘
saved: C:\work\obsidian\herdr_test\Projects\plan.md             Ctrl+H 단축키
```

왼쪽은 편집기, 오른쪽은 볼트 목록과 폴더·노트 목록, 아래는 상태줄입니다. Agents 사이드바 블록이 아니라 pane 전체를 쓰는 터미널 앱입니다.

## 설치

빌드한 뒤 `herdr plugin link .`으로 플러그인을 연결합니다(`herdr-plugin.toml`은 저장소 루트에 있습니다). 또는 herdr pane에서 바이너리를 직접 실행합니다.

```sh
git clone https://github.com/vrwolf1004/herdr-obsidian.git
cd herdr-obsidian
cargo build --release
target/release/herdr-obsidian "C:\work\obsidian\herdr_test"
```

## 액션

| 액션 | 기능 |
| --- | --- |
| `open` | 볼트 앱을 pane에 엽니다. 볼트 인자가 없으면 `config.toml`의 `vaults` 목록을 씁니다 |

Windows에서는 액션 ID 뒤에 `-windows`를 붙입니다. 앱에서 `Ctrl+O`는 볼트의 폴더 열기, `Ctrl+N` / `Ctrl+G`는 노트 / 폴더 만들기, `Ctrl+H`는 전체 단축키 보기입니다. [사용법](docs/usage.ko.md)을 참고하세요.

## 요구 사항

- Windows, Linux, macOS (개발과 테스트는 Windows에서 했습니다)
- herdr 0.9.0 이상
- 빌드용 Rust (stable)

## 문서

- [사용법: 볼트 목록, 단축키, 저장 (한국어)](docs/usage.ko.md)
- [Usage (English)](docs/usage.md)
- [릴리스 노트](docs/release-notes.ko.md)
- [TODO](docs/todo.md)
- [추가 항목](docs/readme-extras.ko.md): README에서 옮긴 항목

## 프로젝트 응원하기

프로젝트가 도움이 되셨다면 커피 한 잔으로 개발을 응원해 주세요.

[![Ko-fi에서 응원하기](https://img.shields.io/badge/Ko--fi-%EC%9D%91%EC%9B%90%ED%95%98%EA%B8%B0-FF5E5B?style=for-the-badge&logo=buymeacoffee&logoColor=white)](https://ko-fi.com/dev_ohyt)
