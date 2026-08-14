local card={api_version=1,id="ICC_903",name="Sanguine Reveler",text="<b>Battlecry:</b> Destroy a friendly minion and gain +2/+2.",set="ICECROWN",type="minion",class="warlock",rarity="common",cost=1,attack=1,health=1,tags={"undead"},keywords={"battlecry"},target_mode="required_if_available"}
function card.targets(ctx,self)local t={}for _,e in ipairs(ctx:friendly_minions(self))do if e~=self then t[#t+1]=e end end return t end
function card.on_battlecry(ctx,self,target)if target then ctx:destroy(target);ctx:continue_with("revel")end end
function card.revel(ctx,self)if ctx:entity(self).zone=="board"then ctx:buff(self,2,2)end end
return card
