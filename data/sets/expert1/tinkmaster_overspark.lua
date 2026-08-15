local card={api_version=1,id="EX1_083",name="Tinkmaster Overspark",text="[x]<b>Battlecry:</b> Transform\nanother random minion\ninto a 5/5 Devilsaur\n or a 1/1 Squirrel.",set="EXPERT1",type="minion",rarity="legendary",cost=3,attack=3,health=3,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local r={};for _,e in ipairs(ctx:minions())do if e~=self then r[#r+1]=e end end;if #r>0 then ctx:random_entity(r,"tink_target")end end
function card.tink_target(ctx,self,target)ctx:set_data(self,"tink_target",target);ctx:random_value({"EX1_tk28","EX1_tk29"},"tink_form")end
function card.tink_form(ctx,self,id)local target=ctx:get_data(self,"tink_target");if target and ctx:entity(target).zone=="board"then ctx:transform(target,id)end end
card.tokens={{id="EX1_tk28",name="Squirrel",text="",set="EXPERT1",type="minion",collectible=false,cost=1,attack=1,health=1,tags={"beast"}},{id="EX1_tk29",name="Devilsaur",text="",set="EXPERT1",type="minion",collectible=false,cost=5,attack=5,health=5,tags={"beast"}}}
return card
