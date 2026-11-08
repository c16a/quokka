//! Traditional Chinese. A string not here falls back to American English.

use super::Translation;

pub struct ZhHant;

impl Translation for ZhHant {
    fn language_tags(&self) -> &'static [&'static str] {
        &["zh-Hant", "zh-TW", "zh-HK", "zh-MO"]
    }

    fn native_name(&self) -> &'static str {
        "繁體中文"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("名稱")
    }
    fn year(&self) -> Option<&'static str> {
        Some("年份")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("加入日期")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("專輯")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("沒有符合這些篩選條件的專輯。")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("尚無專輯。請在「設定」中選擇資料庫檔案夾。")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("所有類型")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("所有年份")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("所有專輯")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("未知的專輯")
    }
    fn play(&self) -> Option<&'static str> {
        Some("播放")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("隨機播放")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("從喜好項目中移除")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("加入喜好項目")
    }
    fn album(&self) -> Option<&'static str> {
        Some("專輯")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("尚無藝人。請在「設定」中選擇資料庫檔案夾。")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("藝人")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("所有藝人")
    }
    // format.rs
    fn unavailable(&self) -> Option<&'static str> {
        Some("無法使用")
    }
    fn mono(&self) -> Option<&'static str> {
        Some("單聲道")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("立體聲")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("加入檔案夾以開始使用。")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("正在同步電台列表…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("電台同步意外停止。")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("掃描意外停止")
    }
    fn video(&self) -> Option<&'static str> {
        Some("影片")
    }
    fn search(&self) -> Option<&'static str> {
        Some("搜尋")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("最近")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("播放列表")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("電台")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("電影")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("設定")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("正在掃描…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("已儲存 TMDb 密鑰。")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("無法儲存設定：{0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("電台：{0} 個。")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("正在掃描 {0}…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("正在掃描 {0}（另有 {1} 個在排隊）…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}：加入或更改 {1} 個，未更改 {2} 個，移除 {3} 個。")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("在 TMDb 上配對了 {0} 個。")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("TMDb 尋找已停止（{0}）；下次掃描時會重試。")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("找不到專輯 {0}")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("無法加入電台：{0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("無法移除電台：{0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("無法加入播放列表：{0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0} 已在資料庫中。")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("已移除 {0}（{1}）。")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("無法移除 {0}：{1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("隱藏側邊欄")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("顯示側邊欄")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("尚無電影。請在「設定」中選擇資料庫檔案夾。")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("電影")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("長度不明")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("從頭播放")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("電影不對？選擇其他電影")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("移除詳細資訊")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("檔案資訊")
    }
    fn source(&self) -> Option<&'static str> {
        Some("來源")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("正在尋找詳細資訊…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("在 TMDb 上尋找詳細資訊")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("請在「設定」中加入你的 TMDb API 密鑰，以查看簡介和演員。")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("簡介")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("演員")
    }
    fn director(&self) -> Option<&'static str> {
        Some("導演")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("編劇")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("製作人員")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("這是哪部電影？")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("請選擇與你的檔案相符的電影。選擇之前不會儲存任何內容。")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("都不是")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("未配對。")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("就是這部")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("尋找意外停止")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("此電影沒有 TMDb 詳細資訊。")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("未知")
    }
    fn container(&self) -> Option<&'static str> {
        Some("容器")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("編解碼器")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("色彩深度")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("色度抽樣")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("動態範圍")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("解析度")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("長寬比")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("影格速率")
    }
    fn length(&self) -> Option<&'static str> {
        Some("長度")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("檔案大小")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("MPEG 傳輸串流")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("MPEG 節目串流")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("原始 AV1 串流（OBU）")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("沒有可用的音訊輸出裝置")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("無法連線至電台。")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("電台連線一再中斷。請按播放重試。")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("暫停")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("正在連線…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("已暫停")
    }
    fn live(&self) -> Option<&'static str> {
        Some("直播")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("沒有可用的音訊輸出裝置")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("倒轉 10 秒（左方向鍵）")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("暫停（空白鍵）")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("播放（空白鍵）")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("快轉 10 秒（右方向鍵）")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("結束全螢幕（Esc）")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("全螢幕（F）")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("取消靜音")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("靜音")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("正在載入影片…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("上一首")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("下一首")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("重複播放")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("未在播放")
    }
    fn output(&self) -> Option<&'static str> {
        Some("輸出")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("系統預設值")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("待播清單")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("歌詞")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some("播放速度。按一下以更改。1× 以外的速度也會改變音高。")
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("歌曲資訊")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("新播放列表名稱")
    }
    fn create(&self) -> Option<&'static str> {
        Some("建立")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("尚無播放列表。請從專輯或搜尋結果加入歌曲。")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("重新命名")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("刪除播放列表")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("所有播放列表")
    }
    fn save(&self) -> Option<&'static str> {
        Some("儲存")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("取消")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("播放播放列表")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("隨機播放播放列表")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("待播清單")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("待播清單是空的")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("清除")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("你加入待播清單的歌曲會顯示在這裡。")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("接著播放")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("加入待播清單")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("加入播放列表")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("尚無播放列表")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("尚無電台。")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("立即同步")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("正在同步…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("你的電台")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("在下方尋找電台並收聽，它會顯示在這裡。")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("尋找電台")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("請同步電台列表以進行搜尋。")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("沒有相符的電台。")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("國家或地區：任何")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("任何國家或地區")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("編解碼器：任何")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("任何編解碼器")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("位元率：任何")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("任何位元率")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("最熱門優先")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("已暫停")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("正在直播")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("停止收聽")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("依名稱、國家或地區、語言或類型搜尋")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("網路電台")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("剛剛")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("最近加入")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("你加入資料庫的專輯會顯示在這裡。")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("最近播放")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("你播放過的專輯會顯示在這裡。")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("搜尋專輯、歌曲、播放列表和電台")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "搜尋專輯、歌曲、播放列表或電台。篩選條件：artist:、album:、genre:、year:、format:flac。",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("電台")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("歌曲")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("音樂")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("移除")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("重新掃描")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("內容：")
    }
    fn library(&self) -> Option<&'static str> {
        Some("資料庫")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("尚無檔案夾。")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("全部重新掃描")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("正在處理")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("未設定類型")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("請選擇每個檔案夾包含的內容。")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("電視節目")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("即將推出")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("加入檔案夾")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some("每個檔案夾只包含一種媒體。其中其他類型的檔案會被略過。")
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("電視節目 · 即將推出")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("選擇檔案夾…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("為資料庫選擇檔案夾")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("外觀")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("主題")
    }
    fn language(&self) -> Option<&'static str> {
        Some("語言")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("系統（{0}）")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("深色")
    }
    fn light(&self) -> Option<&'static str> {
        Some("淺色")
    }
    fn system(&self) -> Option<&'static str> {
        Some("系統")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("播放")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("音量平衡")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "將歌曲調整為相同的響度。帶有 ReplayGain 標籤的檔案會使用這些標籤；其他檔案會在播放時於背景測量，在此之前維持原本的音量。",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("專輯")
    }
    fn track(&self) -> Option<&'static str> {
        Some("歌曲")
    }
    fn off(&self) -> Option<&'static str> {
        Some("關閉")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("交叉淡入淡出")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("每首歌曲會在這段秒數內轉換到下一首。設為零即關閉。")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("在 lrclib.net 上尋找同步歌詞")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "適用於旁邊沒有 .lrc 檔案的歌曲。會將每首歌曲的標題、藝人、專輯和長度傳送至 lrclib.net。找到的歌詞會快取在資料庫中。.lrc 檔案一律優先。",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("電影詳細資訊")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "電影的簡介和演員來自 TMDb。請輸入你自己的 TMDb API 密鑰以使用這些資訊。密鑰會儲存在你的設定檔中，只有你的使用者帳號可讀取，且只會傳送至 TMDb。",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("TMDb API 密鑰")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("儲存密鑰")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("移除密鑰")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("已儲存一組密鑰。")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some("未儲存密鑰。加入密鑰之前，電影不會顯示簡介或演員。")
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("本產品使用 TMDB API，但未經 TMDB 認可或認證。")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("未在播放。")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("此歌曲的標籤中沒有歌詞，旁邊也沒有 .lrc 檔案。")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("正在 lrclib.net 上尋找歌詞…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("在 lrclib.net 上找不到此歌曲的歌詞。")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("再試一次")
    }
    fn title(&self) -> Option<&'static str> {
        Some("標題")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("藝人")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("長度")
    }
    fn size(&self) -> Option<&'static str> {
        Some("大小")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("位元率")
    }
    fn format(&self) -> Option<&'static str> {
        Some("格式")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("聲道")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("圖片")
    }
    fn file(&self) -> Option<&'static str> {
        Some("檔案")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("在 Finder 中顯示")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("標籤")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("回復")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("回復檔案首次編輯之前的標籤")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "要在 lrclib.net 上尋找同步歌詞嗎？你打開的每首歌曲的標題、藝人、專輯和長度都會傳送至該網站。",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("開啟歌詞尋找")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("照片：{0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0} 小時 {1} 分鐘")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0} 分鐘")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("從 {0} 繼續")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("無法取得詳細資訊：{0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("平均 {0} fps，可變")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("影片需要 libmpv：{0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("沒有音訊輸出：{0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("無法播放電台：{0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("無法連線至電台：{0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("無法切換輸出：{0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("無法播放影片：{0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("播放提前停止：{0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("無法播放 {0}：{1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("目錄中有 {0} 個電台 · 同步於{1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("已收聽 {0} 個")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  已收聽")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("位元率：{0}+ kbps")
    }
    fn never(&self) -> Option<&'static str> {
        Some("從未")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("{0} 分鐘前")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("{0} 小時前")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("{0} 天前")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("沒有「{0}」的結果。")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("無法連線至 lrclib.net：{0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("電台同步失敗：{0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}：無法掃描（{1}）。")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("{0} 部電影")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0} 部電影")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("{0} 張專輯")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0} 張專輯")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("{0} 首歌曲")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} 首歌曲")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0} 個檔案夾")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} 個檔案夾")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0} 個檔案")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} 個檔案")
    }
}
