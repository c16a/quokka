//! Japanese. A string not here falls back to American English.

use super::Translation;

pub struct JaJp;

impl Translation for JaJp {
    fn language_tags(&self) -> &'static [&'static str] {
        &["ja-JP", "ja"]
    }

    fn native_name(&self) -> &'static str {
        "日本語"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("名前")
    }
    fn year(&self) -> Option<&'static str> {
        Some("年")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("追加日")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("アルバム")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("このフィルタに一致するアルバムはありません。")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("まだアルバムがありません。「設定」でライブラリのフォルダを選択してください。")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("すべてのジャンル")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("すべての年")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("すべてのアルバム")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("不明なアルバム")
    }
    fn play(&self) -> Option<&'static str> {
        Some("再生")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("シャッフル")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("お気に入りから削除")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("お気に入りに追加")
    }
    fn album(&self) -> Option<&'static str> {
        Some("アルバム")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("まだアーティストがいません。「設定」でライブラリのフォルダを選択してください。")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("アーティスト")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("すべてのアーティスト")
    }
    // format.rs
    fn mono(&self) -> Option<&'static str> {
        Some("モノラル")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("ステレオ")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("フォルダを追加して始めましょう。")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("ラジオ局のリストを同期中…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("ラジオ局の同期が予期せず停止しました。")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("スキャンが予期せず停止しました")
    }
    fn video(&self) -> Option<&'static str> {
        Some("ビデオ")
    }
    fn search(&self) -> Option<&'static str> {
        Some("検索")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("最近")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("プレイリスト")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("ラジオ")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("映画")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("設定")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("スキャン中…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("TMDbキーを保存しました。")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("設定を保存できませんでした: {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("ラジオ: {0}局")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("{0}をスキャン中…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("{0}をスキャン中（ほかに{1}件が待機中）…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}: 追加・変更 {1}、変更なし {2}、削除 {3}。")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("TMDbで{0}件を照合しました。")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("TMDbの検索が停止しました（{0}）。次回のスキャンで再試行します。")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("アルバム「{0}」が見つかりませんでした")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("ラジオ局を追加できませんでした: {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("ラジオ局を削除できませんでした: {0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("プレイリストに追加できませんでした: {0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0}はすでにライブラリにあります。")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("{0}を削除しました（{1}）。")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("{0}を削除できませんでした: {1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("サイドバーを隠す")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("サイドバーを表示")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("まだ映画がありません。「設定」でライブラリのフォルダを選択してください。")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("映画")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("長さ不明")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("最初から再生")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("違う映画ですか？ 別の映画を選択")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("詳細を削除")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("ファイル情報")
    }
    fn source(&self) -> Option<&'static str> {
        Some("ソース")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("詳細を検索中…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("TMDbで詳細を検索")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("あらすじとキャストを表示するには、「設定」でTMDbのAPIキーを追加してください。")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("あらすじ")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("キャスト")
    }
    fn director(&self) -> Option<&'static str> {
        Some("監督")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("脚本")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("スタッフ")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("これはどの映画ですか？")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("ファイルに合う映画を選んでください。選ぶまでは何も保存されません。")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("どれでもない")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("未照合のままにしました。")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("これ")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("検索が予期せず停止しました")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("この映画のTMDbの詳細はありません。")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("不明")
    }
    fn container(&self) -> Option<&'static str> {
        Some("コンテナ")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("コーデック")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("色深度")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("クロマサブサンプリング")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("ダイナミックレンジ")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("解像度")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("アスペクト比")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("フレームレート")
    }
    fn length(&self) -> Option<&'static str> {
        Some("長さ")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("ファイルサイズ")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("MPEGトランスポートストリーム")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("MPEGプログラムストリーム")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("AV1生ストリーム（OBU）")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("使用できるオーディオ出力装置がありません")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("ラジオ局に接続できませんでした。")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("ラジオ局の接続が何度も切れます。もう一度試すには再生を押してください。")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("一時停止")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("接続中…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("一時停止中")
    }
    fn live(&self) -> Option<&'static str> {
        Some("ライブ")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("使用できるオーディオ出力装置がありません")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("10秒戻る（左矢印）")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("一時停止（スペース）")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("再生（スペース）")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("10秒進む（右矢印）")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("フルスクリーンを終了（Esc）")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("フルスクリーン（F）")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("消音解除")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("消音")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("ビデオを読み込み中…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("前のトラック")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("次のトラック")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("リピート")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("再生中の項目はありません")
    }
    fn output(&self) -> Option<&'static str> {
        Some("出力")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("システムのデフォルト")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("次はこちら")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("歌詞")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some("再生速度。クリックで変更します。1×以外の速度では音程も変わります。")
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("トラック情報")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("新規プレイリストの名前")
    }
    fn create(&self) -> Option<&'static str> {
        Some("作成")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("まだプレイリストがありません。アルバムや検索結果からトラックを追加してください。")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("名前を変更")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("プレイリストを削除")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("すべてのプレイリスト")
    }
    fn save(&self) -> Option<&'static str> {
        Some("保存")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("キャンセル")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("プレイリストを再生")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("プレイリストをシャッフル")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("次はこちら")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("キューは空です")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("消去")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("キューに追加したトラックがここに表示されます。")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("次に再生")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("キューに追加")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("プレイリストに追加")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("まだプレイリストがありません")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("まだラジオ局がありません。")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("今すぐ同期")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("同期中…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("あなたのラジオ局")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("下でラジオ局を探して聴いてみましょう。ここに表示されます。")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("ラジオ局を探す")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("検索するにはラジオ局のリストを同期してください。")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("一致するラジオ局はありません。")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("国: すべて")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("すべての国")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("コーデック: すべて")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("すべてのコーデック")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("ビットレート: すべて")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("すべてのビットレート")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("人気順")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("一時停止中")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("ライブ配信中")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("聴くのをやめる")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("名前、国、言語、ジャンルで検索")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("インターネットラジオ")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("たった今")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("最近追加した項目")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("ライブラリに追加したアルバムがここに表示されます。")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("最近再生した項目")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("再生したアルバムがここに表示されます。")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("アルバム、曲、プレイリスト、ラジオ局を検索")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "アルバム、曲、プレイリスト、ラジオ局を検索します。フィルタ: artist:、album:、genre:、year:、format:flac",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("ラジオ局")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("曲")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("ミュージック")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("削除")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("再スキャン")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("内容:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("ライブラリ")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("まだフォルダがありません。")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("すべて再スキャン")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("処理中")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("種類が未設定")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("それぞれのフォルダの内容を選択してください。")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("テレビ番組")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("近日対応")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("フォルダを追加")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some(
            "各フォルダには1種類のメディアを入れます。それ以外の種類のファイルはスキップされます。",
        )
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("テレビ番組 · 近日対応")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("フォルダを選択…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("ライブラリのフォルダを選択")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("外観")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("テーマ")
    }
    fn language(&self) -> Option<&'static str> {
        Some("言語")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("システム（{0}）")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("ダーク")
    }
    fn light(&self) -> Option<&'static str> {
        Some("ライト")
    }
    fn system(&self) -> Option<&'static str> {
        Some("システム")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("再生")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("音量の自動調整")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "トラックの音量を揃えます。ReplayGainタグのあるファイルはそれを使います。ほかのファイルは再生中にバックグラウンドで測定され、それまでは元の音量のままです。",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("アルバム")
    }
    fn track(&self) -> Option<&'static str> {
        Some("トラック")
    }
    fn off(&self) -> Option<&'static str> {
        Some("オフ")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("クロスフェード")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("各トラックはこの秒数をかけて次のトラックへ移ります。0でオフになります。")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("lrclib.netで同期歌詞を検索")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "横に.lrcファイルがないトラックが対象です。各トラックのタイトル、アーティスト、アルバム、長さをlrclib.netに送信します。見つかった歌詞はライブラリに保存されます。.lrcファイルが常に優先されます。",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("映画の詳細")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "映画のあらすじとキャストはTMDbから取得します。使用するには自分のTMDb APIキーを入力してください。キーは設定ファイルに保存され、あなたのユーザアカウントだけが読み取れ、TMDbにのみ送信されます。",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("TMDb APIキー")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("キーを保存")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("キーを削除")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("キーが保存されています。")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some(
            "キーが保存されていません。キーを追加するまで、映画のあらすじやキャストは表示されません。",
        )
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some(
            "この製品はTMDB APIを使用していますが、TMDBによる推奨や認定を受けたものではありません。",
        )
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("再生中の項目はありません。")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("このトラックには、タグにも横の.lrcファイルにも歌詞がありません。")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("lrclib.netで歌詞を検索中…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("lrclib.netでこのトラックの歌詞は見つかりませんでした。")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("再試行")
    }
    fn title(&self) -> Option<&'static str> {
        Some("タイトル")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("アーティスト")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("再生時間")
    }
    fn size(&self) -> Option<&'static str> {
        Some("サイズ")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("ビットレート")
    }
    fn format(&self) -> Option<&'static str> {
        Some("フォーマット")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("チャンネル")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("画像")
    }
    fn file(&self) -> Option<&'static str> {
        Some("ファイル")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Finderに表示")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("タグ")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("元に戻す")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("最初に編集する前のタグに戻します")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "lrclib.netで同期歌詞を検索しますか？ 開いた各トラックのタイトル、アーティスト、アルバム、長さが送信されます。",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("歌詞の検索をオンにする")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("写真: {0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0}時間{1}分")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0}分")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("{0}から再開")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("詳細を取得できませんでした: {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("平均{0} fps、可変")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("ビデオにはlibmpvが必要です: {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("オーディオ出力がありません: {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("ラジオ局を再生できませんでした: {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("ラジオ局に接続できませんでした: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("出力を切り替えられませんでした: {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("ビデオを再生できませんでした: {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("再生が途中で停止しました: {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("{0}を再生できませんでした: {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("ディレクトリ内のラジオ局: {0} · 同期: {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0}局")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  登録済み")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("ビットレート: {0} kbps以上")
    }
    fn never(&self) -> Option<&'static str> {
        Some("なし")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("{0}分前")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("{0}時間前")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("{0}日前")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("「{0}」の結果はありません。")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("lrclib.netに接続できませんでした: {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("ラジオ局の同期に失敗しました: {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}: スキャンできませんでした（{1}）。")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("{0}本の映画")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0}本の映画")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("{0}枚のアルバム")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0}枚のアルバム")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("{0}曲")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0}曲")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0}個のフォルダ")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0}個のフォルダ")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0}個のファイル")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0}個のファイル")
    }
}
