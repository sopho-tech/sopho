import { expect, type APIRequestContext } from "@playwright/test";
import {
  EMAIL_ARRIVAL_TIMEOUT_MILLISECONDS,
  MAILPIT_BASE_URL,
  SOPHO_BASE_URL,
} from "e2e/support/constants";

type MailpitMessageSummary = { ID: string };
type MailpitSearchResult = { messages: MailpitMessageSummary[] };

export type ReceivedEmail = {
  Subject: string;
  Text: string;
};

async function findMessageId(
  request: APIRequestContext,
  query: string,
): Promise<string | null> {
  const response = await request.get(
    `${MAILPIT_BASE_URL}/api/v1/search?query=${encodeURIComponent(query)}`,
  );
  expect(response.status(), await response.text()).toBe(200);
  const searchResult = (await response.json()) as MailpitSearchResult;
  const [newestMessage] = searchResult.messages;
  return newestMessage ? newestMessage.ID : null;
}

export async function waitForEmail(
  request: APIRequestContext,
  recipient: string,
  subject: string,
): Promise<ReceivedEmail> {
  const query = `to:"${recipient}" subject:"${subject}"`;
  await expect
    .poll(() => findMessageId(request, query), {
      message: `an email to ${recipient} about "${subject}"`,
      timeout: EMAIL_ARRIVAL_TIMEOUT_MILLISECONDS,
    })
    .not.toBeNull();
  const messageId = await findMessageId(request, query);
  if (messageId === null) {
    throw new Error(`the email to ${recipient} about "${subject}" vanished`);
  }
  const response = await request.get(
    `${MAILPIT_BASE_URL}/api/v1/message/${messageId}`,
  );
  expect(response.status(), await response.text()).toBe(200);
  return (await response.json()) as ReceivedEmail;
}

export function linkInEmail(email: ReceivedEmail, linkPath: string): string {
  const linkStart = `${SOPHO_BASE_URL}${linkPath}`;
  const link = email.Text.split(/\s+/).find((word) =>
    word.startsWith(linkStart),
  );
  if (!link) {
    throw new Error(
      `no link starting with ${linkStart} in "${email.Subject}":\n${email.Text}`,
    );
  }
  return link;
}

export function tokenInLink(link: string): string {
  const token = new URLSearchParams(new URL(link).hash.slice(1)).get("token");
  if (!token) {
    throw new Error(`no token in ${link}`);
  }
  return token;
}
