import { useMemo, useState } from "react";
import Board, { BH, BW } from "./Board";
import { TUTORIALS, type TutorialChapter, type TutorialLesson, type TutorialPage } from "../data/tutorials";
import { sqFromIccs } from "../lib/types";

interface Props {
  studyFen: string | null;
  onStartStudy: (fen: string) => void;
}

const SCALE = 0.44;

function MiniBoard({ page }: { page: TutorialPage }) {
  const arrow = useMemo(() => {
    if (!page.arrow) return null;
    const [from, to] = sqFromIccs(page.arrow);
    return { from, to };
  }, [page.arrow]);
  return (
    <div className="tut-board" style={{ width: BW * SCALE, height: BH * SCALE }}>
      <div style={{ transform: `scale(${SCALE})`, transformOrigin: "top left", width: BW, height: BH }}>
        <Board
          fen={page.fen!}
          lastMove={null}
          selected={null}
          legalTargets={[]}
          checkSquare={null}
          hint={null}
          bestArrow={arrow}
          flip={false}
          interactive={false}
          showLegal={false}
          onSquareClick={() => {}}
        />
      </div>
      <span className="tut-arrow-label">{page.arrow ? "▲ 关键着法" : ""}</span>
    </div>
  );
}

export default function TutorialPanel({ studyFen, onStartStudy }: Props) {
  const [chapterId, setChapterId] = useState<string>(TUTORIALS[0].id);
  const [lessonId, setLessonId] = useState<string>(TUTORIALS[0].lessons[0].id);
  const [pageIdx, setPageIdx] = useState(0);
  const [reading, setReading] = useState(false);

  const chapter: TutorialChapter = TUTORIALS.find((c) => c.id === chapterId) ?? TUTORIALS[0];
  const lesson: TutorialLesson = chapter.lessons.find((l) => l.id === lessonId) ?? chapter.lessons[0];
  const page: TutorialPage = lesson.pages[Math.min(pageIdx, lesson.pages.length - 1)];

  const openLesson = (ch: TutorialChapter, ls: TutorialLesson) => {
    setChapterId(ch.id);
    setLessonId(ls.id);
    setPageIdx(0);
    setReading(true);
  };

  // 跨课翻页
  const flatLessons = useMemo(() => TUTORIALS.flatMap((c) => c.lessons.map((l) => ({ c, l }))), []);
  const flatIdx = flatLessons.findIndex((x) => x.l.id === lesson.id);
  const goPage = (d: number) => {
    const next = pageIdx + d;
    if (next >= 0 && next < lesson.pages.length) {
      setPageIdx(next);
      return;
    }
    const nl = flatIdx + d;
    if (nl >= 0 && nl < flatLessons.length) {
      const { c, l } = flatLessons[nl];
      openLesson(c, l);
      setPageIdx(d > 0 ? 0 : Math.max(0, l.pages.length - 1));
    }
  };

  if (!reading) {
    return (
      <div className="panel-col">
        <div className="panel-head">
          <span>象棋教程</span>
          <span className="panel-sub">从入门到实战</span>
        </div>
        <div className="tut-list">
          {TUTORIALS.map((c) => (
            <div key={c.id} className="tut-chapter">
              <div className="tut-ch-title">
                <span className="tut-ico">{c.icon}</span> {c.name}
                <span className="tut-ch-desc">{c.desc}</span>
              </div>
              {c.lessons.map((l) => (
                <button key={l.id} className="tut-lesson" onClick={() => openLesson(c, l)}>
                  <i className="tut-dot" />
                  {l.name}
                  <span className="tut-pages">{l.pages.length} 页</span>
                </button>
              ))}
            </div>
          ))}
        </div>
        {studyFen && (
          <div className="tut-studying">
            研究模式进行中 —— 在大棋盘上自由试走，随时回来继续学习。
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="panel-col">
      <div className="panel-head">
        <button className="btn tiny" onClick={() => setReading(false)}>
          ← 目录
        </button>
        <span className="tut-path">
          {chapter.name} · {lesson.name}
        </span>
      </div>
      <div className="tut-viewer">
        <div className="tut-title">{page.title}</div>
        {page.fen && <MiniBoard page={page} />}
        <div className="tut-text">
          {page.text.map((t, i) => (
            <p key={i}>{t}</p>
          ))}
        </div>
        {page.tip && <div className="tut-tip">💡 {page.tip}</div>}
        {page.fen && (
          <button
            className={`btn primary tut-study-btn ${studyFen === page.fen ? "studying" : ""}`}
            onClick={() => onStartStudy(page.fen!)}
          >
            {studyFen === page.fen ? "正在研究此局面 →" : "在棋盘上研究"}
          </button>
        )}
      </div>
      <div className="tut-nav">
        <button className="btn" onClick={() => goPage(-1)} disabled={flatIdx === 0 && pageIdx === 0}>
          上一页
        </button>
        <span className="tut-pageno">
          {pageIdx + 1} / {lesson.pages.length}
        </span>
        <button
          className="btn"
          onClick={() => goPage(1)}
          disabled={flatIdx === flatLessons.length - 1 && pageIdx === lesson.pages.length - 1}
        >
          下一页
        </button>
      </div>
    </div>
  );
}
