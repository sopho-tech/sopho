export function isMac(): boolean {
  if (typeof navigator === "undefined") {
    return false;
  }

  if ("userAgentData" in navigator) {
    const userAgentData = navigator.userAgentData as { platform?: string };
    return userAgentData.platform?.toUpperCase().includes("MAC") ?? false;
  }

  return /Mac|iPhone|iPod|iPad/i.test(navigator.userAgent);
}
