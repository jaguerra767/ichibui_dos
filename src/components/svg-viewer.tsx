import React from 'react';
import logo from '../assets/caldo-icon-blue.svg'

interface SvgViewerProps {
    base64svg: string;
}


const SvgViewer: React.FC<SvgViewerProps> = ({base64svg}) => {
    return base64svg ? <img src={base64svg} alt="SVG"/> : <img src={logo} alt="SVG" className='h-56' />
};

export default SvgViewer;