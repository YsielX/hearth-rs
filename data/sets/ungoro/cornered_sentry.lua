local card={api_version=1,id="UNG_926",name="Cornered Sentry",text="<b>Taunt</b>. <b>Battlecry:</b> Summon three 1/1 Raptors for your opponent.",set="UNGORO",type="minion",class="warrior",rarity="rare",cost=2,attack=2,health=6,tags={"draenei"},keywords={"taunt","battlecry"}}
function card.on_battlecry(ctx,self) local p=ctx:opponent(ctx:controller(self)); for _=1,3 do ctx:summon(p,"UNG_076t1") end end
return card
