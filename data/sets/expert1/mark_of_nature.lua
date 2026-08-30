local card={api_version=1,id="EX1_155",name="Mark of Nature",text="<b>Choose One -</b> Give a minion +4 Attack; or +4 Health and <b>Taunt</b>.",set="EXPERT1",type="spell",class="druid",rarity="common",spell_school="nature",cost=3,keywords={"choose_one"},target_mode="required",targets=function(ctx)return ctx:minions()end}
function card.on_choose_one(ctx,self,target)ctx:set_data(self,"mark_target",target);ctx:choose_options(ctx:controller(self),"Choose One",{{label="+4 Attack",value=1},{label="+4 Health and Taunt",value=2}},"chosen")end
function card.chosen(ctx,self,c)local t=ctx:get_data(self,"mark_target");if c==1 then cardlib.effects.buff(ctx, t,4,0)else cardlib.effects.buff(ctx, t,0,4);cardlib.effects.grant_keyword(ctx, t,"taunt")end end
function card.on_choose_multiple(ctx,self,target)cardlib.effects.buff(ctx, target,4,4);cardlib.effects.grant_keyword(ctx, target,"taunt")end
return card
