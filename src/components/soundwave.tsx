// The MIT Licensed code used in this React FC was
// was taken from: https://github.com/GriffinJohnston/ldrs 
// 🤙 Shoutout to Griffin Johnston for his excellent loading graphics

import React from 'react';

interface SoundWaveProps {
  size?: number;
  color?: string;
  speed?: number;
  strokeWidth?: number;
}

const SoundWave: React.FC<SoundWaveProps> = ({
  size = 150,
  color = 'white',
  speed = 1,
  strokeWidth = 3.5,
}) => {
  return (
    <div
      className="sound-wave-container"
      style={{
        '--uib-size': `${size}px`,
        '--uib-color': color,
        '--uib-speed': `${speed}s`,
        '--uib-stroke': `${strokeWidth}px`,
      } as React.CSSProperties}
    >
      <div className="sound-wave-bar"></div>
      <div className="sound-wave-bar"></div>
      <div className="sound-wave-bar"></div>
      <div className="sound-wave-bar"></div>
      <style>{`
        .sound-wave-container {
          --uib-size: 35px;
          --uib-color: black;
          --uib-speed: 1s;
          --uib-stroke: 3.5px;
          display: flex;
          align-items: center;
          justify-content: space-between;
          width: var(--uib-size);
          height: calc(var(--uib-size) * 0.9);
        }
        .sound-wave-bar {
          width: var(--uib-stroke);
          height: 100%;
          background-color: var(--uib-color);
          transition: background-color 0.3s ease;
        }
        .sound-wave-bar:nth-child(1) {
          animation: grow var(--uib-speed) ease-in-out calc(var(--uib-speed) * -0.45)
            infinite;
        }
        .sound-wave-bar:nth-child(2) {
          animation: grow var(--uib-speed) ease-in-out calc(var(--uib-speed) * -0.3)
            infinite;
        }
        .sound-wave-bar:nth-child(3) {
          animation: grow var(--uib-speed) ease-in-out calc(var(--uib-speed) * -0.15)
            infinite;
        }
        .sound-wave-bar:nth-child(4) {
          animation: grow var(--uib-speed) ease-in-out infinite;
        }
        @keyframes grow {
          0%,
          100% {
            transform: scaleY(0.3);
          }
          50% {
            transform: scaleY(1);
          }
        }
      `}</style>
    </div>
  );
};

export default SoundWave;