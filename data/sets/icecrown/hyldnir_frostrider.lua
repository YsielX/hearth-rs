local card={api_version=1,id="ICC_855",name="Hyldnir Frostrider",text="<b>Battlecry:</b> <b>Freeze</b> your other minions.",set="ICECROWN",type="minion",rarity="common",cost=3,attack=4,health=4,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)for _,e in ipairs(ctx:friendly_minions(self))do if e~=self then ctx:freeze(e)end end end
return card
