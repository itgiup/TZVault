// src/components/Dial.tsx
//
// Signature element: vòng khóa số kiểu két sắt cơ khí.
// spinning=true khi đang xử lý (unlock/setup), đứng yên khi idle.

interface DialProps {
  spinning?: boolean;
  variant?: 'neutral' | 'success' | 'error';
}

export function Dial({ spinning = false, variant = 'neutral' }: DialProps) {
  const indicatorColor =
    variant === 'success' ? 'var(--success)' : variant === 'error' ? 'var(--danger)' : 'var(--brass)';

  const ticks = Array.from({ length: 24 }, (_, i) => {
    const angle = (i / 24) * 360;
    const isMajor = i % 6 === 0;
    return { angle, isMajor };
  });

  return (
    <svg
      className={`dial${spinning ? ' spinning' : ''}`}
      viewBox="0 0 96 96"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <circle cx="48" cy="48" r="44" fill="none" className="dial-ring" strokeWidth="1.5" />
      <circle cx="48" cy="48" r="34" fill="none" className="dial-ring" strokeWidth="1" opacity="0.5" />

      {ticks.map(({ angle, isMajor }, i) => {
        const rad = (angle * Math.PI) / 180;
        const outerR = 44;
        const innerR = isMajor ? 37 : 40;
        const x1 = 48 + outerR * Math.sin(rad);
        const y1 = 48 - outerR * Math.cos(rad);
        const x2 = 48 + innerR * Math.sin(rad);
        const y2 = 48 - innerR * Math.cos(rad);
        return (
          <line
            key={i}
            x1={x1}
            y1={y1}
            x2={x2}
            y2={y2}
            className="dial-ticks"
            strokeWidth={isMajor ? 1.5 : 1}
          />
        );
      })}

      {/* Kim chỉ hướng lên trên - vị trí "khóa" mặc định */}
      <line x1="48" y1="48" x2="48" y2="16" stroke={indicatorColor} strokeWidth="2" strokeLinecap="round" />
      <circle cx="48" cy="48" r="4" className="dial-indicator" fill={indicatorColor} />
    </svg>
  );
}
