import { useEffect } from "react";
import { usePreferencesStore } from "@/modules/settings/preferences";
import { setAnimationScale } from "@/modules/settings/store";

const CSS_VAR = "--animation-speed";

/**
 * Reads the `animationScale` preference and sets a CSS custom property that
 * scales every CSS transition/animation in the app. 0 = instant, 1 = normal,
 * 2 = slow-mo. The variable is applied to :root so all descendants inherit it.
 */
export function useAnimationScale(): void {
  const animationScale = usePreferencesStore((s) => s.animationScale);
  const hydrated = usePreferencesStore((s) => s.hydrated);

  useEffect(() => {
    if (!hydrated) return;
    // scale 0 → multiplier 0 (instant), 1 → 1 (normal), 2 → 2 (slow)
    const multiplier = animationScale;
    document.documentElement.style.setProperty(CSS_VAR, String(multiplier));
  }, [hydrated, animationScale]);
}

export { setAnimationScale };
