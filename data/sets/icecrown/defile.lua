local card={api_version=1,id="ICC_041",name="Defile",text="Deal $1 damage to all minions. If any die, cast this again.",set="ICECROWN",type="spell",class="warlock",rarity="rare",spell_school="shadow",cost=2}
function card.on_play(ctx,self)ctx:set_data(self,"death_count",#ctx:minions_died_this_turn(0)+#ctx:minions_died_this_turn(1));ctx:continue_with("defile_wave")end
function card.defile_wave(ctx,self)local t=ctx:minions();if #t==0 then return end;ctx:damage_all(t,1);ctx:continue_with("defile_check")end
function card.defile_check(ctx,self)local now=#ctx:minions_died_this_turn(0)+#ctx:minions_died_this_turn(1);local old=ctx:get_data(self,"death_count");ctx:set_data(self,"death_count",now);if now>old and #ctx:minions()>0 then ctx:continue_with("defile_wave")end end
return card
