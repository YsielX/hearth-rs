local card={api_version=1,id="NEW1_041",name="Stampeding Kodo",text="<b>Battlecry:</b> Destroy a random enemy minion with 2 or less Attack.",set="EXPERT1",type="minion",rarity="rare",cost=5,attack=3,health=5,tags={"beast"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local r={};for _,e in ipairs(ctx:enemy_minions(self))do if ctx:entity(e).attack<=2 then r[#r+1]=e end end;if #r>0 then ctx:random_entity(r,"kodo_target")end end
function card.kodo_target(ctx,self,target)cardlib.effects.destroy(ctx, target)end
return card
