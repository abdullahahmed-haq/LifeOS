import {
  createContext,
  use,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  commands,
  type AppSettings,
  type ThemePreference,
} from "@lifeos/contracts/bindings";
import type { Locale } from "@lifeos/i18n/catalog";

const presentationKey = "lifeos.presentation.v1";

type Presentation = Pick<
  AppSettings,
  "locale" | "theme" | "timezone" | "weekStartsOn"
>;
type PresentationContextValue = Presentation & {
  ready: boolean;
  setLocale: (locale: Locale) => void;
  setTheme: (theme: ThemePreference) => void;
};

const PresentationContext = createContext<PresentationContextValue | null>(
  null,
);

function defaultPresentation(): Presentation {
  return {
    locale: readPresentation().locale,
    theme: readPresentation().theme,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC",
    weekStartsOn: 1,
  };
}

function readPresentation(): Pick<Presentation, "locale" | "theme"> {
  try {
    const value = JSON.parse(localStorage.getItem(presentationKey) ?? "{}");
    return {
      locale: value.locale === "ar" ? "ar" : "en",
      theme: isTheme(value.theme) ? value.theme : "system",
    };
  } catch {
    return { locale: "en", theme: "system" };
  }
}

function isTheme(value: unknown): value is ThemePreference {
  return value === "light" || value === "dark" || value === "system";
}

function applyPresentation(locale: string, theme: ThemePreference) {
  const direction = locale === "ar" ? "rtl" : "ltr";
  const prefersDark =
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches;
  const resolvedTheme =
    theme === "system" && prefersDark
      ? "dark"
      : theme === "system"
        ? "light"
        : theme;
  document.documentElement.lang = locale;
  document.documentElement.dir = direction;
  document.documentElement.dataset.theme = resolvedTheme;
  document.documentElement.dataset.themePreference = theme;
  localStorage.setItem(presentationKey, JSON.stringify({ locale, theme }));
}

async function unwrap<T>(
  command: Promise<
    { status: "ok"; data: T } | { status: "error"; error: unknown }
  >,
) {
  const result = await command;
  if (result.status === "error") throw result.error;
  return result.data;
}

export function PresentationProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient();
  const [presentation, setPresentation] = useState(defaultPresentation);
  const settings = useQuery({
    queryKey: ["app-settings"],
    queryFn: () => unwrap(commands.appSettings()),
    retry: false,
  });
  const update = useMutation({
    mutationFn: (next: AppSettings) =>
      unwrap(
        commands.updateAppSettings({
          locale: next.locale,
          theme: next.theme,
          timezone: next.timezone,
          weekStartsOn: next.weekStartsOn,
          expectedRevision: next.revision,
          operationId: crypto.randomUUID(),
        }),
      ),
    onSuccess: (receipt) => {
      queryClient.setQueryData(["app-settings"], receipt.data);
      setPresentation(receipt.data);
      applyPresentation(receipt.data.locale, receipt.data.theme);
    },
  });

  useEffect(() => {
    if (!settings.data) return;
    setPresentation(settings.data);
    applyPresentation(settings.data.locale, settings.data.theme);
  }, [settings.data]);

  const value = useMemo<PresentationContextValue>(
    () => ({
      ...presentation,
      ready: settings.isSuccess,
      setLocale: (locale) => {
        applyPresentation(locale, presentation.theme);
        setPresentation((current) => ({ ...current, locale }));
        if (settings.data && locale !== settings.data.locale) {
          update.mutate({ ...settings.data, locale });
        }
      },
      setTheme: (theme) => {
        applyPresentation(presentation.locale, theme);
        setPresentation((current) => ({ ...current, theme }));
        if (settings.data && theme !== settings.data.theme) {
          update.mutate({ ...settings.data, theme });
        }
      },
    }),
    [presentation, settings.data, settings.isSuccess, update],
  );

  return <PresentationContext value={value}>{children}</PresentationContext>;
}

export function usePresentation() {
  const value = use(PresentationContext);
  if (!value) throw new Error("PresentationProvider is required");
  return value;
}

export { applyPresentation };
