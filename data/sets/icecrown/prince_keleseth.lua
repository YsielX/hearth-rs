local card={api_version=1,id="ICC_851",name="Prince Keleseth",text="<b>Battlecry:</b> If your deck has no 2-Cost cards, give all minions in your deck +1/+1.",set="ICECROWN",type="minion",rarity="legendary",cost=2,attack=2,health=2,tags={"undead"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local p=ctx:controller(self);for _,e in ipairs(ctx:deck(p))do if ctx:entity(e).cost==2 then return end end;for _,e in ipairs(ctx:deck(p))do if ctx:entity(e).type=="minion"then ctx:buff(e,1,1)end end end
return card
