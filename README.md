# telegrab
grab a telegraph page, [reference](https://github.com/Artezon/Telegraph-Image-Downloader)

# features
1. dom parse
2. download images
3. archive images into cbz with ComicInfo.xml
4. `ComicInfo.xml` support, Komga compatible

# run
```bash
systemfd --no-pid -s http::9000 -- cargo watch -x "run --bin telegrab"
```