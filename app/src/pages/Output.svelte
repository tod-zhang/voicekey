<script lang="ts">
  import { app, save } from "../lib/store.svelte";
  import { call } from "../lib/api";
  const s = $derived(app.s!);
  const all = $derived(s.channel === "all");
  const win = $derived(app.info!.platform === "win");

  let last = $state("");
  // 设置页开着时也会去别的窗口说话，定时刷新
  $effect(() => {
    const load = () => call<string>("last_target").then((t) => (last = t));
    load();
    const id = setInterval(load, 1500);
    return () => clearInterval(id);
  });
  // macOS 上记录的是 "应用名 (Bundle ID)"，加入列表时取不含空格的 Bundle ID
  const appName = (t: string) => t.match(/\(([^)]+)\)$/)?.[1] ?? t.replace(/\.exe$/, "");
  const listed = $derived(!!last && s.remoteApps.some((a) => a.trim() && last.includes(a.trim().toLowerCase())));
  const split = (v: string) => v.split(/[,，\s]+/).map((a) => a.trim()).filter(Boolean);
  const setApps = (v: string) => save({ remoteApps: split(v) });
  const setPasteApps = (v: string) => save({ pasteApps: split(v) });
</script>

<h2>输出</h2>
<div class="sub">识别结果如何进入输入框。</div>
<div class="group">
  <div class="row">
    <div class="lbl">边说边上屏<small>{all ? "「多渠道」模式下需先挑选结果，此项不可用" : "识别中的文字实时打到光标处，结束后按定稿修正；关闭后改为结束时一次性粘贴"}</small></div>
    <button class="sw" class:on={s.streaming} disabled={all} onclick={() => save({ streaming: !s.streaming })} aria-label="边说边上屏"></button>
  </div>
  {#if win && s.streaming && !all}
    <div class="row">
      <div class="lbl">只粘贴的程序<small>这些程序收不到逐字打出的文字（如微信会把部分字变成标点），改为结束时一次性粘贴；进程名包含其中任一项即生效，用逗号或空格分隔</small></div>
      <input class="txt" value={s.pasteApps.join(", ")} onchange={(e) => setPasteApps(e.currentTarget.value)} />
    </div>
  {/if}
  <div class="row">
    <div class="lbl">胶囊显示实时文字<small>在屏幕底部胶囊中预览识别内容</small></div>
    <button class="sw" class:on={s.liveText} onclick={() => save({ liveText: !s.liveText })} aria-label="胶囊显示实时文字"></button>
  </div>
</div>

<h3>远程桌面</h3>
<div class="group">
  <div class="row">
    <div class="lbl">远程桌面兼容<small>在 UU 远程等远控窗口里说话时，改为先同步剪贴板再自动按 {win ? "Ctrl+V（远端是 Mac 时需把 Ctrl 映射为 ⌘）" : "⌘V"} 上屏，不恢复原剪贴板</small></div>
    <button class="sw" class:on={s.remoteCompat} onclick={() => save({ remoteCompat: !s.remoteCompat })} aria-label="远程桌面兼容"></button>
  </div>
  {#if s.remoteCompat}
    <div class="row">
      <div class="lbl">远控程序<small>{win ? "进程名" : "应用名或 Bundle ID"}包含其中任一项即启用，用逗号或空格分隔</small></div>
      <input class="txt" value={s.remoteApps.join(", ")} onchange={(e) => setApps(e.currentTarget.value)} />
    </div>
    {#if last}
      <div class="row">
        <div class="lbl">上次上屏的程序<small>{last}{listed ? " · 已按远控处理" : ""}</small></div>
        {#if !listed}<button class="btn" onclick={() => save({ remoteApps: [...s.remoteApps, appName(last)] })}>加入远控程序</button>{/if}
      </div>
    {/if}
    <div class="row">
      <div class="lbl">远程边说边上屏（实验）<small>{s.streaming && !all ? "说话时就分段粘贴到远端：文字会一顿一顿出现，识别修正时会删掉重打，期间剪贴板不断变化" : "需先开启上面的「边说边上屏」"}</small></div>
      <button class="sw" class:on={s.remoteStreaming} disabled={!s.streaming || all} onclick={() => save({ remoteStreaming: !s.remoteStreaming })} aria-label="远程边说边上屏"></button>
    </div>
    <div class="row">
      <div class="lbl">粘贴前等待<small>给远控软件同步剪贴板的时间；粘贴出的是旧内容时调大</small></div>
      <input type="range" min="100" max="2000" step="100" value={s.remoteDelay} oninput={(e) => save({ remoteDelay: +e.currentTarget.value })} />
      <b class="val">{s.remoteDelay} ms</b>
    </div>
  {/if}
</div>

<style>
  .txt { width: 220px; padding: 5px 8px; border: 1px solid var(--line); border-radius: var(--rs); background: var(--card2); color: inherit; font: inherit; }
  input[type="range"] { accent-color: var(--accent); width: 160px; }
  .val { width: 64px; }
</style>
