use qqbot_sdk_core::MaterialImagePage;
use serde_json::json;

#[test]
fn material_total_accepts_official_numeric_string_and_integer() {
    for total in [json!("0"), json!(0), json!("12"), json!(12)] {
        let page: MaterialImagePage = serde_json::from_value(json!({
            "images": [], "next_cursor": "", "total": total,
        }))
        .expect("素材列表 total 应兼容官方返回的数字字符串与整数");
        assert!(matches!(page.total, 0 | 12));
    }
}

#[test]
fn material_total_rejects_invalid_or_overflowing_values() {
    for total in [
        json!("-1"),
        json!(-1),
        json!("18446744073709551616"),
        json!(1.5),
        json!("invalid"),
    ] {
        assert!(
            serde_json::from_value::<MaterialImagePage>(json!({
                "images": [], "next_cursor": "", "total": total,
            }))
            .is_err(),
            "无效 total 不应被转换为合法素材数量"
        );
    }
}

#[test]
fn material_entries_accept_official_timestamp_strings() {
    for timestamp in [json!(1794932397), json!("1794932397")] {
        let page: MaterialImagePage = serde_json::from_value(json!({
            "total": "1", "next_cursor": "", "images": [{
                "image_id": "test-image", "url": "qqbot-media://test-image",
                "file_name": "test.png", "preview_url": "https://example.com/test.png",
                "created_at": timestamp, "expires_at": timestamp,
            }],
        }))
        .expect("素材时间戳应兼容官方返回的数字字符串与整数");
        assert_eq!(page.images[0].created_at, 1794932397);
        assert_eq!(page.images[0].expires_at, 1794932397);
    }
}
