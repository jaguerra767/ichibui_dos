import React from 'react';
import logo from './assets/caldo-logo-full-horizontal-white.svg';
import home from './assets/home.svg';

import {DispenseType, User} from './types.ts'
import SettingsMenu from './settings-menu.tsx';
import { Button } from './components/ui/button.tsx';
import { useNavigate, useLocation } from 'react-router-dom';

interface SettingsMenuProps{
    user: User
    currentDispenseType: DispenseType,
    setDispenseType: (dispenseType: DispenseType) => void
}

const Header: React.FC<SettingsMenuProps> = ({user, currentDispenseType, setDispenseType}) => {
    const sudoLoggedIn = user === User.Admin || user === User.Manager || user === User.Operator;
    const navigate = useNavigate();
    const location = useLocation();
    const isHome = location.pathname === '/';
    return (
        <header className='bg-slate-950 absolute top-0 left-0 w-full'>
            <img src={logo} alt="Caldo logo" className='absolute top-2 h-16 w-full'/>
                <div className='absolute top-4 right-0 h-12'>
                    {sudoLoggedIn && <SettingsMenu currentDispenseType={currentDispenseType} setDispenseType={setDispenseType} currentUser={user}/>}
                </div>
                <div className='absolute top-4 left-0 h-12'>
                    {!isHome && <Button className='bg-slate-950 hover:bg-slate-950 h-20 active:bg-slate-950 focus:outline-none focus:ring-0 border-0' onClick={() => navigate('/')}>
                        <img className='h-10' src={home} alt='home'></img>
                    </Button>}
                </div>
        </header>
    );
}

export default Header;

