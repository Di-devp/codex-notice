import { invoke } from "@tauri-apps/api/core";
import type {
  ChannelConfig,
  DashboardSummary,
  EventFilter,
  HookPreview,
  HookStatus,
  NoticeEvent,
  Pagination,
  PendingApproval,
  TrafficWidgetStatus,
  AppLocale,
} from "./types";

export const api = {
  dashboard: () => invoke<DashboardSummary>("get_dashboard_summary"),
  events: (filter: EventFilter, pagination: Pagination) =>
    invoke<NoticeEvent[]>("list_events", { filter, pagination }),
  clearEvents: () => invoke<void>("clear_events"),
  refreshRuntimeStatus: () => invoke<TrafficWidgetStatus>("refresh_runtime_status"),
  appLocale: () => invoke<AppLocale>("get_app_locale"),
  setAppLocale: (locale: AppLocale) => invoke<AppLocale>("set_app_locale", { locale }),
  autostartEnabled: () => invoke<boolean>("get_autostart_enabled"),
  setAutostartEnabled: (enabled: boolean) =>
    invoke<boolean>("set_autostart_enabled", { enabled }),
  channelConfig: () => invoke<ChannelConfig>("get_channel_config"),
  saveFeishuConfig: (webhookUrl: string, signSecret?: string) =>
    invoke<ChannelConfig>("save_feishu_config", { webhookUrl, signSecret }),
  setFeishuEnabled: (enabled: boolean) =>
    invoke<ChannelConfig>("set_feishu_enabled", { enabled }),
  testFeishu: () => invoke<string>("test_feishu_channel"),
  hookStatus: () => invoke<HookStatus>("get_hook_status"),
  previewHookInstall: () => invoke<HookPreview>("preview_hook_install"),
  installHooks: () => invoke<HookStatus>("install_codex_hooks"),
  uninstallHooks: () => invoke<HookStatus>("uninstall_codex_hooks"),
  approvals: () => invoke<PendingApproval[]>("list_pending_approvals"),
  resolveApproval: (id: string, decision: "approved" | "rejected") =>
    invoke<void>("resolve_approval", { id, decision }),
  trafficStatus: () => invoke<TrafficWidgetStatus>("get_traffic_widget_status"),
  setTrafficWidgetEnabled: (enabled: boolean) =>
    invoke<TrafficWidgetStatus>("set_traffic_widget_enabled", { enabled }),
  setTrafficWidgetAlwaysOnTop: (alwaysOnTop: boolean) =>
    invoke<TrafficWidgetStatus>("set_traffic_widget_always_on_top", { alwaysOnTop }),
};
