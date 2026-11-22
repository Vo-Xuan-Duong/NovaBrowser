//! Temporary DOM-level input adapter.
//! This is not native trusted pointer/keyboard/IME dispatch.

use obscura_browser::Page;
use crate::viewport::Viewport;

pub fn click(page: &mut Page, viewport: Viewport, x: f32, y: f32) {
    let Some((x, y)) = viewport.point(x, y) else { return; };
    let script = format!(
        "(function() {{\
            const el = document.elementFromPoint({x}, {y});\
            if (!el) return;\
            const init = {{bubbles:true,cancelable:true,clientX:{x},clientY:{y},button:0}};\
            el.dispatchEvent(new MouseEvent('mousedown', init));\
            if (typeof el.focus === 'function') el.focus();\
            el.dispatchEvent(new MouseEvent('mouseup', init));\
            if (typeof el.click === 'function') el.click();\
        }})()"
    );
    page.evaluate(&script);
}

pub fn scroll(page: &mut Page, viewport: Viewport, x: f32, y: f32, dx: f32, dy: f32) {
    let Some((x, y)) = viewport.point(x, y) else { return; };
    if !dx.is_finite() || !dy.is_finite() { return; }
    let dx = -dx.clamp(-1500.0, 1500.0);
    let dy = -dy.clamp(-1500.0, 1500.0);
    let script = format!(
        "(function() {{\
            const target = document.elementFromPoint({x}, {y}) || document.body;\
            if (!target) return;\
            const wheel = new WheelEvent('wheel', {{bubbles:true,cancelable:true,clientX:{x},clientY:{y},deltaX:{dx},deltaY:{dy},deltaMode:0}});\
            if (!target.dispatchEvent(wheel)) return;\
            const root = document.scrollingElement || document.documentElement || document.body;\
            let el = target;\
            while (el && el !== root && el !== document.body) {{\
                let style;\
                try {{ style = getComputedStyle(el); }} catch (_) {{ break; }}\
                const overflow = style.overflowY === 'auto' || style.overflowY === 'scroll';\
                if (overflow && el.scrollHeight > el.clientHeight) {{ el.scrollBy(dx, dy); return; }}\
                el = el.parentElement;\
            }}\
            window.scrollBy(dx,dy);\
        }})()"
    );
    page.evaluate(&script);
}

pub fn keyboard_script(text: &str) -> Option<String> {
    if text.is_empty() || text.chars().any(char::is_control) {
        return None;
    }
    let json_text = serde_json::to_string(text).ok()?;
    let js = r#"(function() {
        const field = document.activeElement;
        if (!field || (field.tagName !== 'INPUT' && field.tagName !== 'TEXTAREA')) return;
        if (field.disabled || field.readOnly) return;
        const input = __TEXT__;
        const back = input === '__NOVA_BACKSPACE__';
        const del = input === '__NOVA_DELETE__';
        if (input === '__NOVA_ENTER__' && field.tagName === 'INPUT') {
            const keydown = new KeyboardEvent('keydown', {key:'Enter',bubbles:true,cancelable:true});
            if (!field.dispatchEvent(keydown)) return;
            if (field.form && typeof field.form.requestSubmit === 'function') field.form.requestSubmit();
            return;
        }
        const text = input === '__NOVA_ENTER__' ? '\n' : (back || del ? '' : input);
        const old = String(field.value || '');
        const start = typeof field.selectionStart === 'number' ? field.selectionStart : old.length;
        const end = typeof field.selectionEnd === 'number' ? field.selectionEnd : start;
        let from = Math.min(start, end), to = Math.max(start, end);
        if (back && from === to && from > 0) {
            from--;
            if (from > 0) {
                const last = old.charCodeAt(from), prev = old.charCodeAt(from-1);
                if (last >= 0xDC00 && last <= 0xDFFF && prev >= 0xD800 && prev <= 0xDBFF) from--;
            }
        }
        if (del && from === to && to < old.length) {
            to++;
            if (to < old.length) {
                const first = old.charCodeAt(to-1), next = old.charCodeAt(to);
                if (first >= 0xD800 && first <= 0xDBFF && next >= 0xDC00 && next <= 0xDFFF) to++;
            }
        }
        if ((back || del) && from === to) return;
        const kind = back ? 'deleteContentBackward' : (del ? 'deleteContentForward' : 'insertText');
        const before = new InputEvent('beforeinput', {bubbles:true,cancelable:true,data:text,inputType:kind});
        if (!field.dispatchEvent(before)) return;
        const value = old.slice(0,from) + text + old.slice(to);
        if (typeof globalThis.__obscura_setFieldValue === 'function') {
            globalThis.__obscura_setFieldValue(field, 'value', value);
        } else {
            field.value = value;
        }
        if (typeof field.setSelectionRange === 'function') {
            const position = from + text.length;
            field.setSelectionRange(position,position);
        }
        field.dispatchEvent(new InputEvent('input', {bubbles:true,data:text,inputType:kind}));
    })()"#;
    Some(js.replace("__TEXT__", &json_text))
}

pub fn key(page: &mut Page, text: &str) {
    if let Some(script) = keyboard_script(text) {
        page.evaluate(&script);
    }
}

#[cfg(test)]
mod tests {
    use super::keyboard_script;

    #[test]
    fn quotes_are_json_escaped() {
        let js = keyboard_script("\";alert(1)//").unwrap();
        assert!(js.contains(r#"const input = "\";alert(1)//";"#));
    }

    #[test]
    fn special_keys_and_invalid_control_characters() {
        for key in ["__NOVA_BACKSPACE__", "__NOVA_DELETE__", "__NOVA_ENTER__", "ỡ"] {
            assert!(keyboard_script(key).is_some());
        }
        assert!(keyboard_script("").is_none());
        assert!(keyboard_script("\u{0000}").is_none());
    }
}
