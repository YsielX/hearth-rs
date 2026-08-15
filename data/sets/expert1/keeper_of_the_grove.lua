local card={api_version=1,id="EX1_166",name="Keeper of the Grove",text="<b>Choose One -</b> Deal 2 damage; or <b>Silence</b> a minion.",set="EXPERT1",type="minion",class="druid",rarity="rare",cost=4,attack=2,health=4,keywords={"choose_one"}}
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{label="Deal 2 damage",value=1},{label="Silence a minion",value=2}},"chosen")end
function card.chosen(ctx,self,c)ctx:choose_entities(ctx:controller(self),c==1 and "Choose a character"or"Choose a minion",c==1 and ctx:characters()or ctx:minions(),c==1 and "damage_selected"or"silence_selected")end
function card.damage_selected(ctx,self,target)ctx:damage(target,2)end
function card.silence_selected(ctx,self,target)ctx:silence(target)end
function card.on_choose_multiple(ctx,self)ctx:choose_entities(ctx:controller(self),"Choose a character to damage",ctx:characters(),"both_damage_selected")end
function card.both_damage_selected(ctx,self,target)ctx:damage(target,2);ctx:choose_entities(ctx:controller(self),"Choose a minion to silence",ctx:minions(),"silence_selected")end
return card
