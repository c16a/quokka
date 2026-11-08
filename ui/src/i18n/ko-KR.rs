//! Korean. A string not here falls back to American English.

use super::Translation;

pub struct KoKr;

impl Translation for KoKr {
    fn language_tags(&self) -> &'static [&'static str] {
        &["ko-KR", "ko"]
    }

    fn native_name(&self) -> &'static str {
        "한국어"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("이름")
    }
    fn year(&self) -> Option<&'static str> {
        Some("연도")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("추가한 날짜")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("앨범")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("이 필터와 일치하는 앨범이 없습니다.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("아직 앨범이 없습니다. 설정에서 보관함 폴더를 선택하십시오.")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("모든 장르")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("모든 연도")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("모든 앨범")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("알 수 없는 앨범")
    }
    fn play(&self) -> Option<&'static str> {
        Some("재생")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("임의 재생")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("즐겨찾기에서 제거")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("즐겨찾기에 추가")
    }
    fn album(&self) -> Option<&'static str> {
        Some("앨범")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("아직 아티스트가 없습니다. 설정에서 보관함 폴더를 선택하십시오.")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("아티스트")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("모든 아티스트")
    }
    // format.rs
    fn mono(&self) -> Option<&'static str> {
        Some("모노")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("스테레오")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("시작하려면 폴더를 추가하십시오.")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("라디오 방송국 목록 동기화 중…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("라디오 방송국 동기화가 예기치 않게 중단되었습니다.")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("스캔이 예기치 않게 중단됨")
    }
    fn video(&self) -> Option<&'static str> {
        Some("비디오")
    }
    fn search(&self) -> Option<&'static str> {
        Some("검색")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("최근 항목")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("플레이리스트")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("라디오")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("영화")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("설정")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("스캔 중…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("TMDb 키를 저장했습니다.")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("설정을 저장할 수 없습니다: {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("라디오: 방송국 {0}개.")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("{0} 스캔 중…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("{0} 스캔 중(대기 중 {1}개 더 있음)…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}: 추가 또는 변경 {1}개, 변경 없음 {2}개, 제거 {3}개.")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("TMDb에서 {0}개 일치.")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("TMDb 검색이 중단되었습니다({0}). 다음 스캔 때 다시 시도합니다.")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("앨범 {0}을(를) 찾을 수 없음")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("방송국을 추가할 수 없습니다: {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("방송국을 제거할 수 없습니다: {0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("플레이리스트에 추가할 수 없습니다: {0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0}은(는) 이미 보관함에 있습니다.")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("{0} 제거됨({1}).")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("{0}을(를) 제거할 수 없습니다: {1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("사이드바 가리기")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("사이드바 보기")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("아직 영화가 없습니다. 설정에서 보관함 폴더를 선택하십시오.")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("영화")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("길이 알 수 없음")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("처음부터 재생")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("다른 영화입니까? 다른 영화 선택")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("세부사항 제거")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("파일 정보")
    }
    fn source(&self) -> Option<&'static str> {
        Some("소스")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("세부사항 찾는 중…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("TMDb에서 세부사항 찾기")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("줄거리와 출연진을 보려면 설정에서 TMDb API 키를 추가하십시오.")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("줄거리")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("출연진")
    }
    fn director(&self) -> Option<&'static str> {
        Some("감독")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("각본")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("제작진")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("어떤 영화입니까?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("파일에 해당하는 영화를 선택하십시오. 선택하기 전에는 아무것도 저장되지 않습니다.")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("해당 없음")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("일치 항목 없이 남겨 두었습니다.")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("이 영화")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("검색이 예기치 않게 중단됨")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("이 영화에 대한 TMDb 세부사항이 없습니다.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("알 수 없음")
    }
    fn container(&self) -> Option<&'static str> {
        Some("컨테이너")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("코덱")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("색 심도")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("크로마 서브샘플링")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("다이내믹 레인지")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("해상도")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("화면비")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("프레임률")
    }
    fn length(&self) -> Option<&'static str> {
        Some("길이")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("파일 크기")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("MPEG 전송 스트림")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("MPEG 프로그램 스트림")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("원시 AV1 스트림(OBU)")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("사용할 수 있는 오디오 출력 장치가 없음")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("방송국에 연결할 수 없습니다.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("방송국 연결이 계속 끊어집니다. 다시 시도하려면 재생을 누르십시오.")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("일시 정지")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("연결 중…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("일시 정지됨")
    }
    fn live(&self) -> Option<&'static str> {
        Some("라이브")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("사용할 수 있는 오디오 출력 장치가 없음")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("10초 뒤로(왼쪽 화살표)")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("일시 정지(스페이스)")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("재생(스페이스)")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("10초 앞으로(오른쪽 화살표)")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("전체 화면 종료(Esc)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("전체 화면(F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("음소거 해제")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("음소거")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("비디오 로드 중…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("이전 트랙")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("다음 트랙")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("반복")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("재생 중인 항목 없음")
    }
    fn output(&self) -> Option<&'static str> {
        Some("출력")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("시스템 기본값")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("다음 재생")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("가사")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some("재생 속도. 클릭하여 변경합니다. 1× 이외의 속도에서는 음높이도 바뀝니다.")
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("트랙 정보")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("새로운 플레이리스트 이름")
    }
    fn create(&self) -> Option<&'static str> {
        Some("생성")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("아직 플레이리스트가 없습니다. 앨범이나 검색 결과에서 트랙을 추가하십시오.")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("이름 변경")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("플레이리스트 삭제")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("모든 플레이리스트")
    }
    fn save(&self) -> Option<&'static str> {
        Some("저장")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("취소")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("플레이리스트 재생")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("플레이리스트 임의 재생")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("다음 재생")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("대기열이 비어 있음")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("지우기")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("대기열에 추가한 트랙이 여기에 표시됩니다.")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("다음에 재생")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("대기열에 추가")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("플레이리스트에 추가")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("아직 플레이리스트가 없음")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("아직 방송국이 없습니다.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("지금 동기화")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("동기화 중…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("나의 방송국")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("아래에서 방송국을 찾아 들어 보십시오. 여기에 표시됩니다.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("방송국 찾기")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("검색하려면 방송국 목록을 동기화하십시오.")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("일치하는 방송국이 없습니다.")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("국가: 모두")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("모든 국가")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("코덱: 모두")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("모든 코덱")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("비트율: 모두")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("모든 비트율")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("인기순")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("일시 정지됨")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("지금 라이브")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("청취 중단")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("이름, 국가, 언어 또는 장르로 검색")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("인터넷 라디오")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("방금")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("최근 추가한 항목")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("보관함에 추가한 앨범이 여기에 표시됩니다.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("최근 재생한 항목")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("재생한 앨범이 여기에 표시됩니다.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("앨범, 노래, 플레이리스트 및 방송국 검색")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "앨범, 노래, 플레이리스트 또는 방송국을 검색합니다. 필터: artist:, album:, genre:, year:, format:flac.",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("방송국")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("노래")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("음악")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("제거")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("다시 스캔")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("내용:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("보관함")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("아직 폴더가 없습니다.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("모두 다시 스캔")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("처리 중")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("종류가 설정되지 않음")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("각 폴더에 들어 있는 내용을 선택하십시오.")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("TV 프로그램")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("지원 예정")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("폴더 추가")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some("각 폴더에는 한 종류의 미디어만 들어갑니다. 그 안의 다른 종류의 파일은 건너뜁니다.")
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("TV 프로그램 · 지원 예정")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("폴더 선택…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("보관함 폴더 선택")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("화면 모드")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("테마")
    }
    fn language(&self) -> Option<&'static str> {
        Some("언어")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("시스템({0})")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("다크")
    }
    fn light(&self) -> Option<&'static str> {
        Some("라이트")
    }
    fn system(&self) -> Option<&'static str> {
        Some("시스템")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("재생")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("음량 평준화")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "트랙의 음량을 같은 크기로 맞춥니다. ReplayGain 태그가 있는 파일은 그 값을 사용하고, 다른 파일은 재생하는 동안 백그라운드에서 측정되며 그때까지는 원래 음량을 유지합니다.",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("앨범")
    }
    fn track(&self) -> Option<&'static str> {
        Some("트랙")
    }
    fn off(&self) -> Option<&'static str> {
        Some("끔")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("크로스페이드")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("각 트랙이 이 시간(초) 동안 다음 트랙으로 넘어갑니다. 0이면 꺼집니다.")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("lrclib.net에서 동기화된 가사 찾기")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "옆에 .lrc 파일이 없는 트랙에 적용됩니다. 각 트랙의 제목, 아티스트, 앨범 및 길이를 lrclib.net으로 보냅니다. 찾은 가사는 보관함에 저장됩니다. .lrc 파일이 항상 우선합니다.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("영화 세부사항")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "영화의 줄거리와 출연진은 TMDb에서 가져옵니다. 사용하려면 자신의 TMDb API 키를 입력하십시오. 키는 설정 파일에 저장되며, 사용자 계정만 읽을 수 있고, TMDb로만 전송됩니다.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("TMDb API 키")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("키 저장")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("키 제거")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("키가 저장되어 있습니다.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some("저장된 키가 없습니다. 키를 추가할 때까지 영화의 줄거리나 출연진이 표시되지 않습니다.")
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("이 제품은 TMDB API를 사용하지만 TMDB의 보증이나 인증을 받지 않았습니다.")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("재생 중인 항목이 없습니다.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("이 트랙은 태그에 가사가 없고 옆에 .lrc 파일도 없습니다.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("lrclib.net에서 가사 찾는 중…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("lrclib.net에서 이 트랙의 가사를 찾지 못했습니다.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("다시 시도")
    }
    fn title(&self) -> Option<&'static str> {
        Some("제목")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("아티스트")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("재생 시간")
    }
    fn size(&self) -> Option<&'static str> {
        Some("크기")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("비트율")
    }
    fn format(&self) -> Option<&'static str> {
        Some("포맷")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("채널")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("사진")
    }
    fn file(&self) -> Option<&'static str> {
        Some("파일")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Finder에서 보기")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("태그")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("되돌리기")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("파일을 처음 편집하기 전의 태그로 되돌립니다")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "lrclib.net에서 동기화된 가사를 찾겠습니까? 여는 각 트랙의 제목, 아티스트, 앨범 및 길이가 전송됩니다.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("가사 찾기 켜기")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("사진: {0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0}시간 {1}분")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0}분")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("{0}부터 이어서 재생")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("세부사항을 가져올 수 없습니다: {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("평균 {0}fps, 가변")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("비디오를 재생하려면 libmpv가 필요합니다: {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("오디오 출력 없음: {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("방송국을 재생할 수 없습니다: {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("방송국에 연결할 수 없습니다: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("출력을 전환할 수 없습니다: {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("비디오를 재생할 수 없습니다: {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("재생이 일찍 중단되었습니다: {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("{0}을(를) 재생할 수 없습니다: {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("디렉토리에 방송국 {0}개 · 동기화: {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0}개 청취 중")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  청취 중")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("비트율: {0}kbps 이상")
    }
    fn never(&self) -> Option<&'static str> {
        Some("안 함")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("{0}분 전")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("{0}시간 전")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("{0}일 전")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("'{0}'에 대한 결과가 없습니다.")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("lrclib.net에 연결할 수 없습니다: {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("라디오 방송국 동기화 실패: {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}: 스캔할 수 없습니다({1}).")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("영화 {0}편")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("영화 {0}편")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("앨범 {0}개")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("앨범 {0}개")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("트랙 {0}개")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("트랙 {0}개")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("폴더 {0}개")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("폴더 {0}개")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("파일 {0}개")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("파일 {0}개")
    }
}
