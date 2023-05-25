Test graph search with:
```
curl -X 'POST' \                   
  'http://localhost:3000/api/gsearch' \
  -H 'accept: application/json; charset=utf-8' \
  -H 'Content-Type: application/json; charset=utf-8' \
  -d '{
  "search_term": "project evaluation project strategy",
  "skip": 0,
  "limit": 0,
  "role": "project manager"
}'
```
This is a rewrite of [Python-based API](https://github.com/applied-knowledge-systems/the-pattern-api/) in Rust