local card={api_version=1,id="EX1_178",name="Ancient of War",text="<b>Choose One -</b>\n+5 Attack; or +5 Health and <b>Taunt</b>.",set="EXPERT1",type="minion",class="druid",rarity="epic",cost=7,attack=5,health=5,keywords={"choose_one"}}
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{label="+5 Attack",value=1},{label="+5 Health and Taunt",value=2}},"chosen")end
function card.chosen(ctx,self,c)if c==1 then ctx:buff(self,5,0)else ctx:buff(self,0,5);ctx:grant_keyword(self,"taunt")end end
function card.on_choose_multiple(ctx,self)ctx:buff(self,5,5);ctx:grant_keyword(self,"taunt")end
return card
