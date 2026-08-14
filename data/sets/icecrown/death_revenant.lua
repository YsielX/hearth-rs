local card={api_version=1,id="ICC_450",name="Death Revenant",text="<b>Battlecry:</b> Gain +1/+1 for each damaged minion.",set="ICECROWN",type="minion",class="warrior",rarity="rare",cost=5,attack=3,health=3,tags={"undead"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local n=0;for _,e in ipairs(ctx:minions())do if ctx:entity(e).damage>0 then n=n+1 end end;if n>0 then ctx:buff(self,n,n)end end
return card
