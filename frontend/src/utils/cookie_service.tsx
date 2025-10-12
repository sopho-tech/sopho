export class CookieService {
  static setCookie(name: string, value: string, days: number) {
    const expires = new Date(Date.now() + days * 864e5).toUTCString();
    document.cookie = `${name}=${encodeURIComponent(value)}; expires=${expires}; path=/`;
  }

  static getCookie(name: string): string | null {
    const cookies = document.cookie.split("; ");
    for (let cookie of cookies) {
      const [cookieName, cookieValue] = cookie.split("=");
      if (cookieName === name) {
        return decodeURIComponent(cookieValue);
      }
    }
    return null;
  }

  static deleteCookie(name: string) {
    this.setCookie(name, "", -1);
  }

  static listCookies(): Record<string, string> {
    const cookies: Record<string, string> = {};
    document.cookie.split("; ").forEach((cookie) => {
      const [name, value] = cookie.split("=");
      cookies[name] = decodeURIComponent(value);
    });
    return cookies;
  }
}
