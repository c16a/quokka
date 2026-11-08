//! Simplified Chinese. A string not here falls back to American English.

use super::Translation;

pub struct ZhHans;

impl Translation for ZhHans {
    fn language_tags(&self) -> &'static [&'static str] {
        &["zh-Hans", "zh-CN", "zh-SG", "zh-MY", "zh"]
    }

    fn native_name(&self) -> &'static str {
        "简体中文"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("名称")
    }
    fn year(&self) -> Option<&'static str> {
        Some("年份")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("添加日期")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("专辑")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("没有符合这些过滤条件的专辑。")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("还没有专辑。请在“设置”中选择资料库文件夹。")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("所有风格")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("所有年份")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("所有专辑")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("未知专辑")
    }
    fn play(&self) -> Option<&'static str> {
        Some("播放")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("随机播放")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("从收藏中移除")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("添加到收藏")
    }
    fn album(&self) -> Option<&'static str> {
        Some("专辑")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("还没有艺人。请在“设置”中选择资料库文件夹。")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("艺人")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("所有艺人")
    }
    // format.rs
    fn unavailable(&self) -> Option<&'static str> {
        Some("不可用")
    }
    fn mono(&self) -> Option<&'static str> {
        Some("单声道")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("立体声")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("添加一个文件夹即可开始。")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("正在同步电台列表…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("电台同步意外停止。")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("扫描意外停止")
    }
    fn video(&self) -> Option<&'static str> {
        Some("视频")
    }
    fn search(&self) -> Option<&'static str> {
        Some("搜索")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("最近")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("播放列表")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("电台")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("电影")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("设置")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("正在扫描…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("已保存 TMDb 密钥。")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("无法保存设置：{0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("电台：{0} 个。")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("正在扫描 {0}…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("正在扫描 {0}（另有 {1} 个在排队）…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}：添加或更改 {1} 个，未更改 {2} 个，移除 {3} 个。")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("在 TMDb 上匹配了 {0} 个。")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("TMDb 查找已停止（{0}）；下次扫描时将重试。")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("找不到专辑 {0}")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("无法添加电台：{0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("无法移除电台：{0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("无法添加到播放列表：{0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0} 已在资料库中。")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("已移除 {0}（{1}）。")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("无法移除 {0}：{1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("隐藏边栏")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("显示边栏")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("还没有电影。请在“设置”中选择资料库文件夹。")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("电影")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("时长未知")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("从头播放")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("电影不对？选择其他电影")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("移除详细信息")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("文件信息")
    }
    fn source(&self) -> Option<&'static str> {
        Some("来源")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("正在查找详细信息…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("在 TMDb 上查找详细信息")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("请在“设置”中添加你的 TMDb API 密钥，以查看简介和演员。")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("简介")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("演员")
    }
    fn director(&self) -> Option<&'static str> {
        Some("导演")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("编剧")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("制作人员")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("这是哪部电影？")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("请选择与你的文件对应的电影。选择之前不会保存任何内容。")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("都不是")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("未匹配。")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("就是这部")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("查找意外停止")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("此电影没有 TMDb 详细信息。")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("未知")
    }
    fn container(&self) -> Option<&'static str> {
        Some("容器")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("编解码器")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("色深")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("色度抽样")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("动态范围")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("分辨率")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("宽高比")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("帧率")
    }
    fn length(&self) -> Option<&'static str> {
        Some("时长")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("文件大小")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("MPEG 传输流")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("MPEG 节目流")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("原始 AV1 流（OBU）")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("没有可用的音频输出设备")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("无法连接到电台。")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("电台连接一再中断。请按播放重试。")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("暂停")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("正在连接…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("已暂停")
    }
    fn live(&self) -> Option<&'static str> {
        Some("直播")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("没有可用的音频输出设备")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("后退 10 秒（左箭头）")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("暂停（空格）")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("播放（空格）")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("前进 10 秒（右箭头）")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("退出全屏（Esc）")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("全屏（F）")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("取消静音")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("静音")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("正在载入视频…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("上一首")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("下一首")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("重复播放")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("未在播放")
    }
    fn output(&self) -> Option<&'static str> {
        Some("输出")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("系统默认")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("待播清单")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("歌词")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some("播放速度。点按以更改。1× 以外的速度也会改变音高。")
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("歌曲信息")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("新播放列表名称")
    }
    fn create(&self) -> Option<&'static str> {
        Some("创建")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("还没有播放列表。请从专辑或搜索结果中添加歌曲。")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("重新命名")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("删除播放列表")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("所有播放列表")
    }
    fn save(&self) -> Option<&'static str> {
        Some("保存")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("取消")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("播放播放列表")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("随机播放播放列表")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("待播清单")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("待播清单为空")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("清除")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("你加入待播清单的歌曲会显示在这里。")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("接下来播放")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("添加到待播清单")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("添加到播放列表")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("还没有播放列表")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("还没有电台。")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("立即同步")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("正在同步…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("你的电台")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("在下方找到电台并收听，它会显示在这里。")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("查找电台")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("请同步电台列表以进行搜索。")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("没有匹配的电台。")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("国家/地区：任意")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("任意国家/地区")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("编解码器：任意")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("任意编解码器")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("比特率：任意")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("任意比特率")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("最热门优先")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("已暂停")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("正在直播")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("停止收听")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("按名称、国家/地区、语言或风格搜索")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("网络电台")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("刚刚")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("最近添加")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("你添加到资料库的专辑会显示在这里。")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("最近播放")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("你播放过的专辑会显示在这里。")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("搜索专辑、歌曲、播放列表和电台")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "搜索专辑、歌曲、播放列表或电台。过滤条件：artist:、album:、genre:、year:、format:flac。",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("电台")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("歌曲")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("音乐")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("移除")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("重新扫描")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("内容：")
    }
    fn library(&self) -> Option<&'static str> {
        Some("资料库")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("还没有文件夹。")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("全部重新扫描")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("正在处理")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("未设定类型")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("请选择每个文件夹包含的内容。")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("电视节目")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("即将推出")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("添加文件夹")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some("每个文件夹只包含一种媒体。其中其他类型的文件会被跳过。")
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("电视节目 · 即将推出")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("选择文件夹…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("为资料库选择文件夹")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("外观")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("主题")
    }
    fn language(&self) -> Option<&'static str> {
        Some("语言")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("系统（{0}）")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("深色")
    }
    fn light(&self) -> Option<&'static str> {
        Some("浅色")
    }
    fn system(&self) -> Option<&'static str> {
        Some("系统")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("播放")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("音量均衡")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "将歌曲调整到相同的响度。带有 ReplayGain 标签的文件使用这些标签；其他文件会在播放时于后台测量，在此之前保持原有音量。",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("专辑")
    }
    fn track(&self) -> Option<&'static str> {
        Some("歌曲")
    }
    fn off(&self) -> Option<&'static str> {
        Some("关闭")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("淡入淡出")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("每首歌曲在这段秒数内过渡到下一首。设为零即关闭。")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("在 lrclib.net 上查找同步歌词")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "适用于旁边没有 .lrc 文件的歌曲。会将每首歌曲的标题、艺人、专辑和时长发送到 lrclib.net。找到的歌词会缓存在资料库中。.lrc 文件始终优先。",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("电影详细信息")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "电影的简介和演员来自 TMDb。请输入你自己的 TMDb API 密钥以使用它们。密钥保存在你的设置文件中，仅你的用户账户可读取，且只会发送到 TMDb。",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("TMDb API 密钥")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("保存密钥")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("移除密钥")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("已保存一个密钥。")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some("未保存密钥。添加密钥前，电影不会显示简介或演员。")
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("本产品使用 TMDB API，但未经 TMDB 认可或认证。")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("未在播放。")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("此歌曲的标签中没有歌词，旁边也没有 .lrc 文件。")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("正在 lrclib.net 上查找歌词…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("在 lrclib.net 上未找到此歌曲的歌词。")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("重试")
    }
    fn title(&self) -> Option<&'static str> {
        Some("标题")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("艺人")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("时长")
    }
    fn size(&self) -> Option<&'static str> {
        Some("大小")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("比特率")
    }
    fn format(&self) -> Option<&'static str> {
        Some("格式")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("声道")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("图片")
    }
    fn file(&self) -> Option<&'static str> {
        Some("文件")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("在访达中显示")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("标签")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("复原")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("恢复文件首次编辑之前的标签")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "要在 lrclib.net 上查找同步歌词吗？你打开的每首歌曲的标题、艺人、专辑和时长都会发送到该网站。",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("打开歌词查找")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("照片：{0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0} 小时 {1} 分钟")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0} 分钟")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("从 {0} 继续")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("无法获取详细信息：{0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("平均 {0} fps，可变")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("视频需要 libmpv：{0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("没有音频输出：{0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("无法播放电台：{0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("无法连接到电台：{0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("无法切换输出：{0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("无法播放视频：{0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("播放提前停止：{0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("无法播放 {0}：{1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("目录中有 {0} 个电台 · 同步于{1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("已收听 {0} 个")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  已收听")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("比特率：{0}+ kbps")
    }
    fn never(&self) -> Option<&'static str> {
        Some("从未")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("{0} 分钟前")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("{0} 小时前")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("{0} 天前")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("没有“{0}”的结果。")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("无法连接 lrclib.net：{0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("电台同步失败：{0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}：无法扫描（{1}）。")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("{0} 部电影")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0} 部电影")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("{0} 张专辑")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0} 张专辑")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("{0} 首歌曲")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} 首歌曲")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0} 个文件夹")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} 个文件夹")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0} 个文件")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} 个文件")
    }
    fn audio_track(&self) -> Option<&'static str> {
        Some("音轨")
    }
    fn track_label(&self) -> Option<&'static str> {
        Some("音轨 {0}")
    }
    fn deinterlace(&self) -> Option<&'static str> {
        Some("去交错")
    }
    fn screenshot_saved(&self) -> Option<&'static str> {
        Some("截图已保存至 {0}")
    }
}
