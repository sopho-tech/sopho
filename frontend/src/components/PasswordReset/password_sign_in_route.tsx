import { data } from "react-router";
import { fetchSignInPageOptions } from "src/api/auth/queries";

export async function requirePasswordSignIn(): Promise<null> {
  const signInPageOptions = await fetchSignInPageOptions();
  if (!signInPageOptions.password_sign_in_available) {
    throw data(null, { status: 404, statusText: "Not Found" });
  }
  return null;
}
