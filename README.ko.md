# herdr-obsidian

[herdr](https://herdr.dev) pane 안에서 [Obsidian](https://obsidian.md) 볼트를 보고, 읽고, 편집하고, 검색합니다. Obsidian 앱 없이 동작하며, Google Drive 데스크톱 앱으로 동기화된 폴더도 볼트로 쓸 수 있습니다.

> 상태: 개발 중. 아래 기능은 계획이거나 일부만 구현되어 있습니다.

## 기능 (계획)

- 볼트 폴더 설정
- 옆 패널에 폴더와 노트 목록 표시
- 목록에서 Markdown 노트를 클릭해 열기
- 노트 편집, 입력하면 자동 저장
- 노트 본문 검색
- 테스트용 herdr tab

## 요구 사항

- herdr 0.9.0 이상
- 소스 빌드 시 Rust (stable)

## 빌드

```sh
cargo build --release
```

## 사용

```sh
herdr-obsidian --help
```

명령은 아직 구현되지 않았습니다. 예정된 사용법은 [docs/usage.ko.md](docs/usage.ko.md)를 보세요.

## 문서

- [한국어 사용법](docs/usage.ko.md)
- [Usage (English)](docs/usage.md)

## 라이선스

MIT
