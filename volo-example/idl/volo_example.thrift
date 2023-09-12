namespace rs volo.example



struct GetItemRequest {
    1: required string op,
    2: required string key,
    3: required string value,
    4: required i32 life,
}

struct GetItemResponse {
    1: required string op,
    2: required string key,
    3: required string value,
    4: required bool state,
}

service ItemService {
    GetItemResponse GetItem (1: GetItemRequest req),
}

