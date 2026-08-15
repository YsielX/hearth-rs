local card={api_version=1,id="EX1_248",name="Feral Spirit",text="Summon two 2/3 Spirit Wolves with <b>Taunt</b>. <b>Overload:</b> (1)",set="EXPERT1",type="spell",class="shaman",rarity="rare",spell_school="nature",cost=3,keywords={"overload"},keyword_params={overload=1},on_play=function(ctx,self)local p=ctx:controller(self);ctx:summon(p,"EX1_tk11");ctx:summon(p,"EX1_tk11")end}
card.tokens={{id="EX1_tk11",name="Spirit Wolf",text="<b>Taunt</b>",set="EXPERT1",type="minion",class="shaman",collectible=false,cost=2,attack=2,health=3,tags={"undead","beast"},keywords={"taunt"}}}
return card
