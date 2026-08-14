local card={api_version=1,id="ICC_904",name="Wicked Skeleton",text="<b>Battlecry:</b> Gain +1/+1 for each minion that died this turn.",set="ICECROWN",type="minion",rarity="common",cost=4,attack=1,health=1,tags={"undead"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local n=#ctx:minions_died_this_turn(0)+#ctx:minions_died_this_turn(1);if n>0 then ctx:buff(self,n,n)end end
return card
