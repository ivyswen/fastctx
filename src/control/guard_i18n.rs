//! Localized copy for provider-derived output protection.

use super::i18n::Language;

/// Complete provider-guard copy for one supported language.
#[derive(Debug)]
pub(crate) struct GuardMessages {
    pub(crate) label: &'static str,
    pub(crate) active_note: &'static str,
    pub(crate) disabled_note: &'static str,
    pub(crate) available_note: &'static str,
    pub(crate) locked_note: &'static str,
    pub(crate) disable_confirm: &'static str,
    pub(crate) budget_follows_guarded_note: &'static str,
}

impl GuardMessages {
    #[cfg(test)]
    fn values(&self) -> [&'static str; 7] {
        [
            self.label,
            self.active_note,
            self.disabled_note,
            self.available_note,
            self.locked_note,
            self.disable_confirm,
            self.budget_follows_guarded_note,
        ]
    }
}

/// Returns provider-guard copy for a supported language.
pub(crate) const fn messages(language: Language) -> &'static GuardMessages {
    match language {
        Language::En => &EN,
        Language::ZhCn => &ZH_CN,
        Language::ZhTw => &ZH_TW,
        Language::Ja => &JA,
        Language::Ko => &KO,
        Language::Es => &ES,
        Language::Fr => &FR,
        Language::De => &DE,
        Language::PtBr => &PT_BR,
        Language::Ru => &RU,
        Language::It => &IT,
        Language::Tr => &TR,
        Language::Pl => &PL,
        Language::Nl => &NL,
        Language::Vi => &VI,
        Language::Id => &ID,
        Language::Uk => &UK,
    }
}

macro_rules! guard_messages {
    ($label:expr, $active:expr, $disabled:expr, $available:expr, $locked:expr,
     $confirm:expr, $budget:expr) => {
        GuardMessages {
            label: $label,
            active_note: $active,
            disabled_note: $disabled,
            available_note: $available,
            locked_note: $locked,
            disable_confirm: $confirm,
            budget_follows_guarded_note: $budget,
        }
    };
}

const EN: GuardMessages = guard_messages!(
    "Provider guard",
    "Local compaction detected. Guarded limits are active for Apply and newly started runtime sessions.",
    "Protection is disabled. Large outputs can make local compaction slow, expensive, or fail.",
    "Protection is enabled. Your selected tier stays effective unless the visible provider requires local compaction.",
    "Guarded is locked at host 10000 and FastCtx 9000. Your selected tier is preserved and returns automatically when remote compaction is available.",
    "Disable provider output protection?\nWithout Guarded limits, large tool outputs can make local compaction slow, expensive, or fail. This affects both Apply and newly started runtime sessions.",
    "auto — follows Guarded's full 100% share."
);
const ZH_CN: GuardMessages = guard_messages!(
    "Provider 输出保护",
    "检测到本地压缩。Apply 与新启动的运行时会话现已启用 Guarded 限制。",
    "保护已关闭。大输出可能让本地压缩变慢、变贵或失败。",
    "保护已开启。除非当前可见的 provider 需要本地压缩，否则仍使用你选择的档位。",
    "Guarded 锁定为宿主 10000、FastCtx 9000。你选择的档位会保留，并在远端压缩可用时自动恢复。",
    "要关闭 provider 输出保护吗？\n没有 Guarded 限制时，大型工具输出可能让本地压缩变慢、变贵或失败。这会同时影响 Apply 和新启动的运行时会话。",
    "auto：跟随 Guarded 的完整 100% 比例。"
);
const ZH_TW: GuardMessages = guard_messages!(
    "Provider 輸出保護",
    "偵測到本機壓縮。Apply 與新啟動的執行階段工作階段已套用 Guarded 限制。",
    "保護已停用。大型輸出可能讓本機壓縮變慢、成本增加或失敗。",
    "保護已啟用。除非目前可見的 provider 需要本機壓縮，否則仍使用所選層級。",
    "Guarded 鎖定為主機 10000、FastCtx 9000。所選層級會保留，並在遠端壓縮可用時自動恢復。",
    "要停用 provider 輸出保護嗎？\n若沒有 Guarded 限制，大型工具輸出可能讓本機壓縮變慢、成本增加或失敗。這會同時影響 Apply 與新啟動的執行階段工作階段。",
    "auto：跟隨 Guarded 的完整 100% 比例。"
);
const JA: GuardMessages = guard_messages!(
    "Provider 出力保護",
    "ローカル圧縮を検出しました。Apply と新しく起動するランタイムセッションに Guarded 制限が有効です。",
    "保護は無効です。大きな出力によりローカル圧縮が遅く、高コストになり、失敗する場合があります。",
    "保護は有効です。表示中の provider がローカル圧縮を必要としない限り、選択した段階が有効です。",
    "Guarded はホスト 10000、FastCtx 9000 に固定されます。選択した段階は保持され、リモート圧縮が使えると自動で戻ります。",
    "provider 出力保護を無効にしますか？\nGuarded 制限がないと、大きなツール出力によりローカル圧縮が遅く、高コストになり、失敗する場合があります。Apply と新しいランタイムセッションの両方に影響します。",
    "auto：Guarded の完全な 100% 割合に従います。"
);
const KO: GuardMessages = guard_messages!(
    "Provider 출력 보호",
    "로컬 압축이 감지되었습니다. Apply와 새 런타임 세션에 Guarded 제한이 적용됩니다.",
    "보호가 꺼져 있습니다. 큰 출력은 로컬 압축을 느리고 비싸게 만들거나 실패시킬 수 있습니다.",
    "보호가 켜져 있습니다. 표시된 provider가 로컬 압축을 요구하지 않으면 선택한 단계가 유지됩니다.",
    "Guarded는 호스트 10000, FastCtx 9000으로 잠깁니다. 선택한 단계는 보존되며 원격 압축이 가능해지면 자동으로 복원됩니다.",
    "provider 출력 보호를 끌까요?\nGuarded 제한이 없으면 큰 도구 출력으로 로컬 압축이 느리고 비싸지거나 실패할 수 있습니다. Apply와 새 런타임 세션 모두에 영향을 줍니다.",
    "auto: Guarded의 전체 100% 비율을 따릅니다."
);
const ES: GuardMessages = guard_messages!(
    "Protección del provider",
    "Se detectó compactación local. Los límites Guarded se aplican a Apply y a las nuevas sesiones de ejecución.",
    "La protección está desactivada. Las salidas grandes pueden volver la compactación local lenta, costosa o hacerla fallar.",
    "La protección está activada. El nivel elegido se mantiene salvo que el provider visible requiera compactación local.",
    "Guarded queda fijado en 10000 para el host y 9000 para FastCtx. El nivel elegido se conserva y vuelve automáticamente cuando hay compactación remota.",
    "¿Desactivar la protección de salida del provider?\nSin los límites Guarded, las salidas grandes pueden volver la compactación local lenta, costosa o hacerla fallar. Afecta a Apply y a las nuevas sesiones de ejecución.",
    "auto — sigue la proporción completa del 100% de Guarded."
);
const FR: GuardMessages = guard_messages!(
    "Protection du provider",
    "Compactage local détecté. Les limites Guarded s’appliquent à Apply et aux nouvelles sessions d’exécution.",
    "La protection est désactivée. Les sorties volumineuses peuvent ralentir, renchérir ou faire échouer le compactage local.",
    "La protection est activée. Le niveau choisi reste actif sauf si le provider visible exige un compactage local.",
    "Guarded est verrouillé à 10000 côté hôte et 9000 côté FastCtx. Le niveau choisi est conservé et revient automatiquement quand le compactage distant est disponible.",
    "Désactiver la protection de sortie du provider ?\nSans les limites Guarded, les grandes sorties peuvent ralentir, renchérir ou faire échouer le compactage local. Cela touche Apply et les nouvelles sessions d’exécution.",
    "auto — suit la part complète de 100 % de Guarded."
);
const DE: GuardMessages = guard_messages!(
    "Provider-Schutz",
    "Lokale Komprimierung erkannt. Guarded-Grenzen gelten für Apply und neu gestartete Laufzeitsitzungen.",
    "Der Schutz ist deaktiviert. Große Ausgaben können lokale Komprimierung langsam, teuer oder fehlerhaft machen.",
    "Der Schutz ist aktiviert. Die gewählte Stufe bleibt aktiv, sofern der sichtbare Provider keine lokale Komprimierung benötigt.",
    "Guarded ist auf Host 10000 und FastCtx 9000 gesperrt. Die gewählte Stufe bleibt erhalten und kehrt bei verfügbarer Remote-Komprimierung automatisch zurück.",
    "Provider-Ausgabeschutz deaktivieren?\nOhne Guarded-Grenzen können große Tool-Ausgaben lokale Komprimierung langsam, teuer oder fehlerhaft machen. Dies betrifft Apply und neue Laufzeitsitzungen.",
    "auto — folgt dem vollen Guarded-Anteil von 100 %."
);
const PT_BR: GuardMessages = guard_messages!(
    "Proteção do provider",
    "Compactação local detectada. Os limites Guarded valem para Apply e novas sessões de execução.",
    "A proteção está desativada. Saídas grandes podem tornar a compactação local lenta, cara ou fazê-la falhar.",
    "A proteção está ativada. O nível escolhido continua ativo, salvo se o provider visível exigir compactação local.",
    "Guarded fica travado em 10000 para o host e 9000 para o FastCtx. O nível escolhido é preservado e retorna automaticamente quando há compactação remota.",
    "Desativar a proteção de saída do provider?\nSem os limites Guarded, saídas grandes podem tornar a compactação local lenta, cara ou fazê-la falhar. Isso afeta Apply e novas sessões de execução.",
    "auto — segue a fração completa de 100% do Guarded."
);
const RU: GuardMessages = guard_messages!(
    "Защита provider",
    "Обнаружено локальное сжатие. Ограничения Guarded действуют для Apply и новых сеансов выполнения.",
    "Защита отключена. Большой вывод может замедлить, удорожить или сорвать локальное сжатие.",
    "Защита включена. Выбранный уровень действует, пока видимый provider не требует локального сжатия.",
    "Guarded фиксирует пределы 10000 для хоста и 9000 для FastCtx. Выбранный уровень сохраняется и автоматически возвращается при доступном удалённом сжатии.",
    "Отключить защиту вывода provider?\nБез ограничений Guarded большой вывод инструментов может замедлить, удорожить или сорвать локальное сжатие. Это влияет на Apply и новые сеансы выполнения.",
    "auto — следует полной доле Guarded 100%."
);
const IT: GuardMessages = guard_messages!(
    "Protezione provider",
    "Rilevata compattazione locale. I limiti Guarded valgono per Apply e per le nuove sessioni di esecuzione.",
    "La protezione è disattivata. Output grandi possono rendere la compattazione locale lenta, costosa o farla fallire.",
    "La protezione è attiva. Il livello scelto resta valido salvo che il provider visibile richieda la compattazione locale.",
    "Guarded è bloccato a 10000 per l’host e 9000 per FastCtx. Il livello scelto viene conservato e torna automaticamente quando è disponibile la compattazione remota.",
    "Disattivare la protezione output del provider?\nSenza i limiti Guarded, output grandi possono rendere la compattazione locale lenta, costosa o farla fallire. Ciò riguarda Apply e le nuove sessioni di esecuzione.",
    "auto — segue la quota completa del 100% di Guarded."
);
const TR: GuardMessages = guard_messages!(
    "Provider koruması",
    "Yerel sıkıştırma algılandı. Guarded sınırları Apply ve yeni çalışma zamanı oturumları için etkindir.",
    "Koruma kapalı. Büyük çıktılar yerel sıkıştırmayı yavaşlatabilir, pahalılaştırabilir veya başarısız kılabilir.",
    "Koruma açık. Görünen provider yerel sıkıştırma gerektirmedikçe seçilen kademe etkin kalır.",
    "Guarded, host için 10000 ve FastCtx için 9000 değerine kilitlenir. Seçilen kademe korunur ve uzaktan sıkıştırma kullanılabildiğinde otomatik döner.",
    "Provider çıktı koruması kapatılsın mı?\nGuarded sınırları olmadan büyük araç çıktıları yerel sıkıştırmayı yavaşlatabilir, pahalılaştırabilir veya başarısız kılabilir. Bu, Apply ve yeni çalışma zamanı oturumlarını etkiler.",
    "auto — Guarded'ın tam %100 payını izler."
);
const PL: GuardMessages = guard_messages!(
    "Ochrona providera",
    "Wykryto lokalną kompakcję. Limity Guarded obowiązują dla Apply i nowych sesji wykonawczych.",
    "Ochrona jest wyłączona. Duże wyjścia mogą spowolnić, podrożyć lub przerwać lokalną kompakcję.",
    "Ochrona jest włączona. Wybrany poziom pozostaje aktywny, chyba że widoczny provider wymaga lokalnej kompakcji.",
    "Guarded jest zablokowany na 10000 dla hosta i 9000 dla FastCtx. Wybrany poziom zostaje zachowany i wraca automatycznie, gdy dostępna jest kompakcja zdalna.",
    "Wyłączyć ochronę wyjścia providera?\nBez limitów Guarded duże wyjścia narzędzi mogą spowolnić, podrożyć lub przerwać lokalną kompakcję. Dotyczy to Apply i nowych sesji wykonawczych.",
    "auto — podąża za pełnym udziałem Guarded 100%."
);
const NL: GuardMessages = guard_messages!(
    "Providerbescherming",
    "Lokale compactie gedetecteerd. Guarded-limieten gelden voor Apply en nieuw gestarte runtimesessies.",
    "Bescherming is uitgeschakeld. Grote uitvoer kan lokale compactie traag, duur of onbetrouwbaar maken.",
    "Bescherming is ingeschakeld. Het gekozen niveau blijft actief tenzij de zichtbare provider lokale compactie vereist.",
    "Guarded is vergrendeld op host 10000 en FastCtx 9000. Het gekozen niveau blijft bewaard en keert automatisch terug zodra externe compactie beschikbaar is.",
    "Provider-uitvoerbescherming uitschakelen?\nZonder Guarded-limieten kan grote tooluitvoer lokale compactie traag, duur of onbetrouwbaar maken. Dit raakt Apply en nieuwe runtimesessies.",
    "auto — volgt het volledige Guarded-aandeel van 100%."
);
const VI: GuardMessages = guard_messages!(
    "Bảo vệ provider",
    "Đã phát hiện nén cục bộ. Giới hạn Guarded áp dụng cho Apply và các phiên chạy mới.",
    "Bảo vệ đã tắt. Đầu ra lớn có thể làm nén cục bộ chậm, tốn kém hoặc thất bại.",
    "Bảo vệ đã bật. Mức đã chọn vẫn có hiệu lực trừ khi provider đang hiển thị cần nén cục bộ.",
    "Guarded khóa ở 10000 cho host và 9000 cho FastCtx. Mức đã chọn được giữ lại và tự động trở lại khi có nén từ xa.",
    "Tắt bảo vệ đầu ra provider?\nNếu không có giới hạn Guarded, đầu ra công cụ lớn có thể làm nén cục bộ chậm, tốn kém hoặc thất bại. Điều này ảnh hưởng Apply và các phiên chạy mới.",
    "auto — theo tỷ lệ đầy đủ 100% của Guarded."
);
const ID: GuardMessages = guard_messages!(
    "Perlindungan provider",
    "Kompaksi lokal terdeteksi. Batas Guarded berlaku untuk Apply dan sesi runtime yang baru dimulai.",
    "Perlindungan dinonaktifkan. Keluaran besar dapat membuat kompaksi lokal lambat, mahal, atau gagal.",
    "Perlindungan diaktifkan. Tingkat pilihan tetap berlaku kecuali provider yang terlihat memerlukan kompaksi lokal.",
    "Guarded dikunci pada host 10000 dan FastCtx 9000. Tingkat pilihan disimpan dan kembali otomatis saat kompaksi jarak jauh tersedia.",
    "Nonaktifkan perlindungan keluaran provider?\nTanpa batas Guarded, keluaran alat yang besar dapat membuat kompaksi lokal lambat, mahal, atau gagal. Ini memengaruhi Apply dan sesi runtime baru.",
    "auto — mengikuti porsi penuh 100% Guarded."
);
const UK: GuardMessages = guard_messages!(
    "Захист provider",
    "Виявлено локальне стиснення. Обмеження Guarded діють для Apply і нових сеансів виконання.",
    "Захист вимкнено. Великий вивід може сповільнити, здорожчити або зірвати локальне стиснення.",
    "Захист увімкнено. Вибраний рівень діє, доки видимий provider не потребує локального стиснення.",
    "Guarded фіксує 10000 для хоста і 9000 для FastCtx. Вибраний рівень зберігається й автоматично повертається, коли доступне віддалене стиснення.",
    "Вимкнути захист виводу provider?\nБез обмежень Guarded великий вивід інструментів може сповільнити, здорожчити або зірвати локальне стиснення. Це впливає на Apply і нові сеанси виконання.",
    "auto — дотримується повної частки Guarded 100%."
);

#[cfg(test)]
mod tests {
    use super::messages;
    use crate::control::i18n::ALL_LANGUAGES;

    #[test]
    fn every_language_has_complete_provider_guard_copy() {
        for language in ALL_LANGUAGES {
            let messages = messages(language);
            assert!(
                messages
                    .values()
                    .iter()
                    .all(|value| !value.trim().is_empty()),
                "{} has an empty provider-guard translation",
                language.code()
            );
            assert!(messages.locked_note.contains("10000"));
            assert!(messages.locked_note.contains("9000"));
            assert!(messages.disable_confirm.contains("Guarded"));
            assert!(messages.budget_follows_guarded_note.contains("100"));
            assert!(messages.budget_follows_guarded_note.contains('%'));
        }
    }
}
