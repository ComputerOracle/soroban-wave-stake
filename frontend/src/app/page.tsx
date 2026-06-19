'use client';

import { useState, useEffect } from 'react';
import { isAllowed, setAllowed, getUserInfo } from '@stellar/freighter-api';

interface Issue {
  id: number;
  title: string;
  repo: string;
  bounty: number;
  stake: number;
  slashedPool: number;
  status: 'Open' | 'Claimed' | 'Resolved';
  deadline: string;
  extensionsUsed: number;
}

export default function Home() {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  
  const [issues, setIssues] = useState<Issue[]>([
    { id: 101, title: "Optimize cross-contract WASM call bindings", repo: "stellar/soroban-cli", bounty: 750, stake: 25, slashedPool: 0, status: 'Open', deadline: '3 days remaining', extensionsUsed: 0 },
    { id: 102, title: "Fix memory leaks inside parallelized event ingestion router", repo: "stellar/soroban-rpc", bounty: 1200, stake: 50, slashedPool: 100, status: 'Open', deadline: 'Passed - Eligible to Slash', extensionsUsed: 0 },
    { id: 103, title: "Implement Graceful Forfeit & Commit Extensions", repo: "ComputerOracle/StellarEscrow", bounty: 1500, stake: 60, slashedPool: 0, status: 'Claimed', deadline: '2 hours remaining (1 claim extension available)', extensionsUsed: 0 }
  ]);

  useEffect(() => {
    checkWalletConnection();
  }, []);

  const checkWalletConnection = async () => {
    try {
      if (await isAllowed()) {
        const info = await getUserInfo();
        setWalletAddress(info.publicKey);
      }
    } catch (err) {
      console.error("Freighter verification failure", err);
    }
  };

  const connectWallet = async () => {
    setLoading(true);
    try {
      await setAllowed();
      await checkWalletConnection();
    } catch (err) {
      alert("User rejected wallet linking authorization.");
    } finally {
      setLoading(false);
    }
  };

  const handleClaim = async (issueId: number) => {
    if (!walletAddress) return alert("Please link your Freighter wallet first.");
    alert(`Invoking claim_issue context on-chain for Issue #${issueId}. Processing micro-stake requirement...`);
    
    setIssues(issues.map(issue => {
      if (issue.id === issueId) {
        return {
          ...issue,
          status: 'Claimed',
          deadline: '24 hours remaining (1 claim extension available)'
        };
      }
      return issue;
    }));
  };

  const handleForfeit = async (issueId: number) => {
    if (!walletAddress) return alert("Please link your Freighter wallet first.");
    const confirmed = confirm("Are you sure you want to gracefully forfeit this claim? You will recover 50% of your total stake, and 50% will go to the issue's slashed pool.");
    if (!confirmed) return;
    
    alert(`Invoking graceful_forfeit context on-chain for Issue #${issueId}...`);
    
    setIssues(issues.map(issue => {
      if (issue.id === issueId) {
        const penalty = (issue.stake * (issue.extensionsUsed > 0 ? 2 : 1)) / 2;
        return {
          ...issue,
          status: 'Open',
          slashedPool: issue.slashedPool + penalty,
          deadline: 'Re-opened'
        };
      }
      return issue;
    }));
  };

  const handleExtension = async (issueId: number) => {
    if (!walletAddress) return alert("Please link your Freighter wallet first.");
    const issue = issues.find(i => i.id === issueId);
    if (issue && issue.extensionsUsed > 0) {
      return alert("This claim has already used its single allowed extension!");
    }
    
    const confirmed = confirm(`Are you sure you want to request a deadline extension? This requires locking up an additional ${issue?.stake} USDC stake (doubling your skin-in-the-game).`);
    if (!confirmed) return;
    
    alert(`Invoking request_extension context on-chain for Issue #${issueId}. Doubling the stake and adding +24 hours to the deadline...`);
    
    setIssues(issues.map(i => {
      if (i.id === issueId) {
        return {
          ...i,
          extensionsUsed: 1,
          deadline: 'Extended: 26 hours remaining'
        };
      }
      return i;
    }));
  };

  return (
    <div className="min-h-screen p-8 max-w-6xl mx-auto flex flex-col justify-between">
      <header className="flex justify-between items-center border-b border-[#30363d] pb-6 mb-10">
        <div>
          <h1 className="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
            🌊 soroban-wave-stake
          </h1>
          <p className="text-sm text-[#8b949e]">The Anti-Ghosting Framework for Open-Source Dev Sprints</p>
        </div>
        {walletAddress ? (
          <div className="flex items-center gap-2 px-4 py-2 bg-[#21262d] border border-[#30363d] rounded-lg text-xs font-mono text-[#58a6ff]">
            <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
            {walletAddress.slice(0, 6)}...{walletAddress.slice(-4)}
          </div>
        ) : (
          <button
            onClick={connectWallet}
            disabled={loading}
            className="px-5 py-2 bg-[#238636] hover:bg-[#2ea043] disabled:opacity-50 text-white rounded-md text-sm font-semibold transition"
          >
            {loading ? "Connecting..." : "Link Wallet (Freighter)"}
          </button>
        )}
      </header>

      <main className="flex-1 space-y-6">
        <div className="bg-[#161b22] border border-[#30363d] rounded-xl p-6 mb-8">
          <h2 className="text-lg font-semibold text-white mb-2">How it works</h2>
          <p className="text-sm text-[#8b949e] leading-relaxed">
            Ecosystem sprint boards face persistent operational challenges with developers claiming issues and then going silent. 
            <strong> Soroban Wave Stake</strong> implements a programmatic skin-in-the-game protocol: lock a tiny crypto stake to claim an exclusive ticket. Complete it to unlock your stake and the bounty reward. If you ghost the team, your stake rolls directly into the target issue pool to fund the next contributor.
          </p>
          <div className="mt-4 p-3.5 bg-[#238636]/10 border border-[#238636]/20 rounded-lg text-xs text-[#58a6ff]">
            <strong>🆕 New Feature Added:</strong> You can now <strong>gracefully forfeit</strong> a ticket before the deadline to recover 50% of your stake (preventing complete slashing). Or, you can <strong>extend your deadline</strong> once by doubling your stake commitment!
          </div>
        </div>

        <h3 className="text-md font-semibold text-white uppercase tracking-wider text-[#8b949e]">Active Sprint Pipeline</h3>
        
        <div className="grid gap-4">
          {issues.map((issue) => (
            <div key={issue.id} className="p-5 bg-[#161b22] border border-[#30363d] rounded-xl flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div className="space-y-1 flex-1">
                <div className="flex items-center gap-3">
                  <span className={`text-xs px-2.5 py-0.5 font-semibold rounded-full border ${
                    issue.status === 'Open' 
                      ? 'bg-[#238636]/20 text-[#3fb950] border-[#238636]/30' 
                      : 'bg-[#58a6ff]/20 text-[#58a6ff] border-[#58a6ff]/30'
                  }`}>
                    {issue.status}
                  </span>
                  <span className="text-xs text-[#8b949e] font-mono">{issue.repo}</span>
                </div>
                <h4 className="text-base font-semibold text-white hover:text-[#58a6ff] cursor-pointer transition">
                  {issue.title} <span className="text-sm font-normal text-[#8b949e]">#{issue.id}</span>
                </h4>
                <p className="text-xs text-[#8b949e] flex items-center gap-1">
                  ⏱️ Deadline terms: <span className={issue.deadline.includes("Passed") ? "text-[#f85149] font-medium" : "text-[#d29922]"}>{issue.deadline}</span>
                </p>
              </div>
              
              <div className="flex items-center justify-between md:justify-end gap-6 border-t md:border-t-0 border-[#30363d] pt-4 md:pt-0">
                <div className="text-left md:text-right min-w-[150px]">
                  <p className="text-lg font-bold text-white">{issue.bounty} USDC</p>
                  <p className="text-xs text-[#8b949e]">
                    Requires {issue.stake * (issue.extensionsUsed > 0 ? 2 : 1)} USDC Stake
                    {issue.extensionsUsed > 0 && <span className="text-[#3fb950] block font-semibold">(Extended)</span>}
                  </p>
                  {issue.slashedPool > 0 && (
                    <p className="text-xs text-[#d29922] font-semibold">+ {issue.slashedPool} USDC Retainer Bonus</p>
                  )}
                </div>

                <div className="flex gap-2">
                  {issue.status === 'Open' ? (
                    <button
                      onClick={() => handleClaim(issue.id)}
                      className="px-4 py-2 bg-[#238636] hover:bg-[#2ea043] text-white border border-transparent rounded-md text-xs font-semibold transition"
                    >
                      Lock Stake & Claim
                    </button>
                  ) : (
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleExtension(issue.id)}
                        disabled={issue.extensionsUsed > 0}
                        className="px-3 py-2 bg-[#21262d] hover:bg-[#30363d] disabled:opacity-50 text-[#d29922] border border-[#30363d] rounded-md text-xs font-semibold transition"
                        title={issue.extensionsUsed > 0 ? "Already extended once" : "Double stake to extend deadline"}
                      >
                        ⏱️ Extend (+24h)
                      </button>
                      <button
                        onClick={() => handleForfeit(issue.id)}
                        className="px-3 py-2 bg-[#f85149]/10 hover:bg-[#f85149]/20 text-[#f85149] border border-[#f85149]/30 rounded-md text-xs font-semibold transition"
                      >
                        🏳️ Forfeit (50%)
                      </button>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </main>

      <footer className="border-t border-[#30363d] mt-16 pt-6 text-center text-xs text-[#8b949e]">
        Built for the Stellar Developer Ecosystem & Drips Wave Sprint Integration Framework. 2026.
      </footer>
    </div>
  );
}
