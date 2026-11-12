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

빌드한 뒤 `herdr plugin install .`으로 플러그인을 설치합니다(`herdr-plugin.toml`은 저장소 루트에 있습니다). 또는 herdr pane에서 바이너리를 직접 실행합니다.

```sh
git clone https://github.com/vrwolf1004/herdr-obsidian.git
cd herdr-obsidian
cargo build --release
target/release/herdr-obsidian "C:\work\obsidian\herdr_test"
```

## 빠른 시작

- `Ctrl+O`: 볼트 안의 폴더를 엽니다.
- `Ctrl+N` / `Ctrl+G`: 노트 / 폴더를 만듭니다.
- `Ctrl+H`: 모든 단축키를 봅니다.

## 문서

- [사용법: 볼트 목록, 단축키, 저장 (한국어)](docs/usage.ko.md)
- [Usage (English)](docs/usage.md)
- [릴리스 노트](docs/release-notes.ko.md)
- [TODO](docs/todo.md)

## 라이선스

MIT

## 프로젝트 응원하기

프로젝트가 도움이 되셨다면 커피 한 잔으로 개발을 응원해 주세요.

[![Ko-fi에서 응원하기](https://img.shields.io/badge/Ko--fi-%EC%9D%91%EC%9B%90%ED%95%98%EA%B8%B0-FF5E5B?style=for-the-badge&logo=buymeacoffee&logoColor=white)](https://ko-fi.com/dev_ohyt)
