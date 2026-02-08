import { useEffect, useState } from "react";
import { SpinnerGap } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";

export function MaintenanceScreen() {
  const { t } = useTranslation();
  const [dots, setDots] = useState("");

  useEffect(() => {
    const dotInterval = setInterval(() => {
      setDots((d) => (d.length >= 3 ? "" : d + "."));
    }, 500);

    const checkInterval = setInterval(async () => {
      try {
        const res = await fetch("/api/v1/system/maintenance");
        const data = await res.json();
        if (!data.maintenance) {
          window.location.href = "/login";
        }
      } catch {
        // Server still down
      }
    }, 5000);

    return () => {
      clearInterval(dotInterval);
      clearInterval(checkInterval);
    };
  }, []);

  return (
    <div className="fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-gray-950/95 backdrop-blur-sm">
      <SpinnerGap weight="bold" className="w-12 h-12 text-primary-400 animate-spin mb-6" />
      <h1 className="text-2xl font-bold text-white mb-2">
        {t("system.updating")}
      </h1>
      <p className="text-gray-400 text-sm">{dots}</p>
    </div>
  );
}
