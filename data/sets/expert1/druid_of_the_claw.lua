local card={api_version=1,id="EX1_165",name="Druid of the Claw",text="[x]<b>Choose One -</b> Transform\ninto a 7/6 with <b>Rush</b>;\nor a 4/9 with <b>Taunt</b>.",set="EXPERT1",type="minion",class="druid",rarity="common",cost=6,attack=4,health=6,keywords={"choose_one"}}
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{label="7/6 with Rush",value=1},{label="4/9 with Taunt",value=2}},"chosen")end
function card.chosen(ctx,self,c)ctx:transform(self,c==1 and "EX1_165t1"or"EX1_165t2")end
function card.on_choose_multiple(ctx,self)ctx:transform(self,"EX1_165t1");ctx:buff(self,0,3);ctx:grant_keyword(self,"taunt")end
card.tokens={{id="EX1_165t1",name="Druid of the Claw",text="<b>Rush</b>",set="EXPERT1",type="minion",class="druid",collectible=false,cost=6,attack=7,health=6,tags={"beast"},keywords={"rush"}},{id="EX1_165t2",name="Druid of the Claw",text="<b>Taunt</b>",set="EXPERT1",type="minion",class="druid",collectible=false,cost=6,attack=4,health=9,tags={"beast"},keywords={"taunt"}}}
return card
