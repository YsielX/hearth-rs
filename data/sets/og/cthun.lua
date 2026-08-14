local card={api_version=1,id="OG_280",name="C'Thun",text="<b>Battlecry:</b> Deal damage\nequal to this minion's\nAttack randomly split among all enemies.",set="OG",type="minion",rarity="legendary",cost=8,attack=6,health=6,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)ctx:set_data(self,"damage_left",math.min(100,ctx:entity(self).attack));ctx:continue_with("fire_cthun")end
function card.fire_cthun(ctx,self)
    if ctx:get_data(self,"damage_left")<=0 then return end
    local pool={}
    for _,enemy in ipairs(ctx:enemy_characters(self))do
        local dormant=false
        for _,keyword in ipairs(ctx:entity(enemy).keywords)do
            if keyword=="dormant"then dormant=true break end
        end
        if not dormant then pool[#pool+1]=enemy end
    end
    if #pool>0 then ctx:random_entity(pool,"hit_enemy")end
end
function card.hit_enemy(ctx,self,target)ctx:damage(target,1);local n=ctx:get_data(self,"damage_left")-1;ctx:set_data(self,"damage_left",n);if n>0 then ctx:continue_with("fire_cthun")end end
return card
