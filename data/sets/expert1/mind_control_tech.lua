local card={api_version=1,id="EX1_085",name="Mind Control Tech",text="[x]<b>Battlecry:</b> If your opponent\nhas 4 or more minions,\ntake control of one.",set="EXPERT1",type="minion",rarity="rare",cost=5,attack=3,health=3,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local r=ctx:enemy_minions(self);if #r>=4 then ctx:random_entity(r,"steal_selected")end end
function card.steal_selected(ctx,self,target)ctx:change_controller(target,ctx:controller(self))end
return card
