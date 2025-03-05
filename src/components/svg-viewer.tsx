import React from 'react';
import logo from '../assets/caldo-icon-blue.svg'

interface SvgViewerProps {
    base64svg: string | undefined;
    height: string | undefined;
}


const SvgViewer: React.FC<SvgViewerProps> = ({base64svg, height}) => {
    return base64svg ? <img src={base64svg} alt="SVG" className={height}/> : <img src={logo} alt="SVG" className='h-56' />
};

export default SvgViewer;