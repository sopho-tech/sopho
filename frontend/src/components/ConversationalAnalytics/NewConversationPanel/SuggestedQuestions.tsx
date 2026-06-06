import { useSuggestedQuestions } from "src/api/suggested_question";
import { SuggestedQuestionList } from "src/components/ConversationalAnalytics/SuggestedQuestionList";

type Props = {
  connectionId: string;
  onPick: (text: string) => void;
};

export function SuggestedQuestions({ connectionId, onPick }: Props) {
  const { data } = useSuggestedQuestions(connectionId);

  return (
    <SuggestedQuestionList
      questions={(data ?? []).map((q) => ({ id: q.id, text: q.question_text }))}
      onPick={onPick}
    />
  );
}
