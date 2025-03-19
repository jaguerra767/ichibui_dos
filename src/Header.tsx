import React from 'react';
import logo from './assets/caldo-logo-full-horizontal-white.svg';
import foodLogo from './assets/FOOD_Logo_rev (1).png';
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
            <div className='flex justify-center items-center gap-2'>
                <img src={logo} alt="Caldo logo" className='h-16'/>
                <div className="pl-10 text-white text-4xl font-bold">x</div> 
                <img src={foodLogo} alt="Caldo logo" className='top-2 h-35 w-60'/>
            </div>
            
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

