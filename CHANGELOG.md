# Rustgram versions

## 0.1.5
- 2022/07/25
- use self instead of ref self for `GramHttpErr` trait

## 0.1.4
- 2022/07/24
- use self instead of ref sef for `HttpResult` trait to convert a value to json

## 0.1.3
- 2022/07/23
- added new Result type for `IntoResponse`
  - users can return a Result with a custom Response creation. Not only for Err but also for Ok 