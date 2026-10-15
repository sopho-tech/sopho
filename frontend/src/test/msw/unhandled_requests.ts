type UnhandledRequest = {
  frame: { getUnhandledMessage: () => Promise<string> };
  defaults: { error: () => void };
};

const unhandledRequestMessages: string[] = [];

export async function recordUnhandledRequest({
  frame,
  defaults,
}: UnhandledRequest) {
  unhandledRequestMessages.push(await frame.getUnhandledMessage());
  defaults.error();
}

export function assertEveryRequestWasHandled() {
  const messages = unhandledRequestMessages.splice(0);
  if (messages.length > 0) {
    throw new Error(
      `The test made requests that no MSW handler answers:\n${messages.join("\n")}`,
    );
  }
}
