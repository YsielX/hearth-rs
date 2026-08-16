local card={api_version=1,id="EX1_154",name="Wrath",text="<b>Choose One -</b>\nDeal $3 damage to a minion; or $1 damage\nand draw a card.",set="EXPERT1",type="spell",class="druid",rarity="common",spell_school="nature",cost=2,keywords={"choose_one"},target_mode="required",targets=function(ctx)return ctx:minions()end}
function card.on_choose_one(ctx,self,target)ctx:set_data(self,"wrath_target",target);ctx:choose_options(ctx:controller(self),"Choose One",{{label="Deal 3 damage",value=1},{label="Deal 1 damage and draw",value=2}},"chosen")end
function card.chosen(ctx,self,c)local t=ctx:get_data(self,"wrath_target");cardlib.effects.damage(ctx, t,c==1 and 3 or 1);if c==2 then ctx:draw(ctx:controller(self),1)end end
function card.on_choose_multiple(ctx,self,target)cardlib.effects.damage(ctx, target,4);ctx:draw(ctx:controller(self),1)end
return card
