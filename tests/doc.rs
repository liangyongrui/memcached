#[macro_use]
extern crate lazy_static;

use async_std::task;
use memcached::Client;

lazy_static! {
    static ref CLIENT: Client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
}

#[async_std::test]
async fn doc_async_test() -> memcached::Result<()> {
    let s1 = task::spawn(async { t1().await.unwrap() });
    let s3 = task::spawn(async { t3().await.unwrap() });
    let s4 = task::spawn(async { t4().await.unwrap() });
    let s8 = task::spawn(async { t8().await.unwrap() });
    let s9 = task::spawn(async { t9().await.unwrap() });
    let s10 = task::spawn(async { t10().await.unwrap() });
    let s11 = task::spawn(async { t11().await.unwrap() });
    let s12 = task::spawn(async { t12().await.unwrap() });
    let s13 = task::spawn(async { t13().await.unwrap() });
    let s14 = task::spawn(async { t14().await.unwrap() });
    let s15 = task::spawn(async { t15().await.unwrap() });
    let s16 = task::spawn(async { t16().await.unwrap() });
    let s17 = task::spawn(async { t17().await.unwrap() });
    let s18 = task::spawn(async { t18().await.unwrap() });
    task::block_on(s1);
    task::block_on(s3);
    task::block_on(s4);
    task::block_on(s8);
    task::block_on(s9);
    task::block_on(s10);
    task::block_on(s11);
    task::block_on(s12);
    task::block_on(s13);
    task::block_on(s14);
    task::block_on(s15);
    task::block_on(s16);
    task::block_on(s17);
    task::block_on(s18);
    // flush op
    let s6 = task::spawn(async { t6().await.unwrap() });
    task::block_on(s6);
    let s7 = task::spawn(async { t7().await.unwrap() });
    task::block_on(s7);

    assert!(memcached::Client::connect_with("", 2, |_| 1).is_err());
    assert!(memcached::Client::connect_with(vec!["".to_owned()], 2, |_| 1).is_err());
    assert!(memcached::Client::connect_with(Vec::<String>::new(), 2, |_| 1).is_err());
    Ok(())
}

async fn t1() -> memcached::Result<()> {
    CLIENT
        .set::<&[u8], _>("abcd", &[1, 2, 3, 4, 5], 100)
        .await?;
    let t: Option<Vec<u8>> = CLIENT.get("abcd").await?;
    assert_eq!(t.unwrap(), vec![1, 2, 3, 4, 5]);
    let t = CLIENT.get::<Vec<u8>, _>("abcd".repeat(100)).await;
    assert!(t.is_err());
    Ok(())
}

async fn t3() -> memcached::Result<()> {
    let version = CLIENT.version().await?;
    dbg!(version);
    Ok(())
}

async fn t4() -> memcached::Result<()> {
    let t: Option<String> = CLIENT.get("get_none").await?;
    assert_eq!(t, None);
    Ok(())
}
async fn t6() -> memcached::Result<()> {
    CLIENT.set("flush_test", "hello", 100).await?;
    CLIENT.flush().await?;
    let t: Option<String> = CLIENT.get("flush_test").await?;
    assert_eq!(t, None);
    Ok(())
}
async fn t7() -> memcached::Result<()> {
    CLIENT.set("flush_with_delay_test", "hello", 100).await?;
    CLIENT.flush_with_delay(2).await?;
    let t: Option<String> = CLIENT.get("flush_with_delay_test").await?;
    assert_eq!(t, Some("hello".to_owned()));
    async_std::task::sleep(core::time::Duration::from_secs(2)).await;
    let t: Option<String> = CLIENT.get("flush_with_delay_test").await?;
    assert_eq!(t, None);
    Ok(())
}
async fn t8() -> memcached::Result<()> {
    CLIENT.delete("add_test").await?;
    CLIENT.add("add_test", "hello", 100).await?;
    // repeat add KeyExists
    CLIENT.add("add_test", "hello233", 100).await.unwrap_err();
    let t: Option<String> = CLIENT.get("add_test").await?;
    assert_eq!(t, Some("hello".to_owned()));
    Ok(())
}
async fn t9() -> memcached::Result<()> {
    CLIENT.delete("replace_test").await?;
    // KeyNotFound
    CLIENT
        .replace("replace_test", "hello", 100)
        .await
        .unwrap_err();
    CLIENT.add("replace_test", "hello", 100).await?;
    CLIENT.replace("replace_test", "hello233", 100).await?;
    let t: Option<String> = CLIENT.get("replace_test").await?;
    assert_eq!(t, Some("hello233".to_owned()));
    Ok(())
}
async fn t10() -> memcached::Result<()> {
    CLIENT.set("append_test", "hello", 100).await?;
    CLIENT.append("append_test", ", 233").await?;
    let t: Option<String> = CLIENT.get("append_test").await?;
    assert_eq!(t, Some("hello, 233".to_owned()));
    Ok(())
}
async fn t11() -> memcached::Result<()> {
    CLIENT.set("prepend_test", "hello", 100).await?;
    CLIENT.prepend("prepend_test", "233! ").await?;
    let t: Option<String> = CLIENT.get("prepend_test").await?;
    assert_eq!(t, Some("233! hello".to_owned()));
    Ok(())
}
async fn t12() -> memcached::Result<()> {
    CLIENT.add("delete_test", "hello", 100).await?;
    let t: Option<String> = CLIENT.get("delete_test").await?;
    assert_eq!(t, Some("hello".to_owned()));
    CLIENT.delete("delete_test").await?;
    let t: Option<String> = CLIENT.get("delete_test").await?;
    assert_eq!(t, None);
    Ok(())
}
async fn t13() -> memcached::Result<()> {
    CLIENT.set("increment_test", 100, 100).await?;
    CLIENT.increment("increment_test", 10).await?;
    assert_eq!(120, CLIENT.increment("increment_test", 10).await.unwrap());
    let t: Option<u64> = CLIENT.get("increment_test").await?;
    assert_eq!(t, Some(120));
    Ok(())
}
async fn t14() -> memcached::Result<()> {
    CLIENT.set("decrement_test", 100, 100).await?;
    let _t = CLIENT.decrement("decrement_test", 10).await?;
    assert_eq!(80, CLIENT.decrement("decrement_test", 10).await.unwrap());
    let t: Option<u64> = CLIENT.get("decrement_test").await?;
    assert_eq!(t.unwrap(), 80);
    Ok(())
}
async fn t15() -> memcached::Result<()> {
    CLIENT.set("touch_test", "100", 100).await?;
    async_std::task::sleep(core::time::Duration::from_secs(1)).await;
    let t: Option<String> = CLIENT.get("touch_test").await?;
    assert_eq!(t, Some("100".to_owned()));
    CLIENT.touch("touch_test", 1).await?;
    async_std::task::sleep(core::time::Duration::from_secs(1)).await;
    let t: Option<String> = CLIENT.get("touch_test").await?;
    assert_eq!(t, None);
    Ok(())
}
async fn t16() -> memcached::Result<()> {
    let t = CLIENT.stats().await?;
    dbg!(t);
    Ok(())
}

async fn t17() -> memcached::Result<()> {
    CLIENT.set("gets_test1", "100", 100).await?;
    CLIENT.set("gets_test2", "200", 100).await?;
    let t = CLIENT
        .gets::<String, _>(&["gets_test1", "gets_test2"])
        .await
        .unwrap();
    dbg!(t);
    Ok(())
}
async fn t18() -> memcached::Result<()> {
    CLIENT.set("cas_test1", "100", 100).await?;
    let t = CLIENT.gets::<String, _>(&["cas_test1"]).await?;
    dbg!(&t);
    let k = t.get("cas_test1").unwrap();
    assert_eq!(&k.0, "100");
    let t = CLIENT
        .cas("cas_test1", "200", 100, k.2.unwrap() - 1)
        .await?;
    dbg!(&t);
    let t = CLIENT.get::<String, _>("cas_test1").await?;
    assert_eq!(t.unwrap(), "100".to_owned());
    let t = CLIENT.cas("cas_test1", "300", 100, k.2.unwrap()).await?;
    dbg!(&t);
    let t = CLIENT.get::<String, _>("cas_test1").await?;
    assert_eq!(t.unwrap(), "300".to_owned());
    Ok(())
}
