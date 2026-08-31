local card={api_version=1,id="EX1_166",name="Keeper of the Grove",text="<b>Choose One -</b> Deal 2 damage; or <b>Silence</b> a minion.",set="EXPERT1",type="minion",class="druid",rarity="rare",cost=4,attack=2,health=4,keywords={"choose_one"}}
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{card_id="EX1_166a",label="Deal 2 damage"},{card_id="EX1_166b",label="Silence a minion"}},"chosen")end
function card.chosen(ctx,self,c)ctx:choose_entities(ctx:controller(self),c=="EX1_166a" and "Choose a character"or"Choose a minion",c=="EX1_166a" and ctx:characters()or ctx:minions(),c=="EX1_166a" and "damage_selected"or"silence_selected")end
function card.damage_selected(ctx,self,target)cardlib.effects.damage(ctx, target,2)end
function card.silence_selected(ctx,self,target)ctx:silence(target)end
function card.on_choose_multiple(ctx,self)ctx:choose_entities(ctx:controller(self),"Choose a character to damage",ctx:characters(),"both_damage_selected")end
function card.both_damage_selected(ctx,self,target)cardlib.effects.damage(ctx, target,2);ctx:choose_entities(ctx:controller(self),"Choose a minion to silence",ctx:minions(),"silence_selected")end
card.tokens={{id="EX1_166a",name="Moonfire",text="Deal 2 damage.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=4},{id="EX1_166b",name="Dispel",text="<b>Silence</b> a minion.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=4}}
return card
