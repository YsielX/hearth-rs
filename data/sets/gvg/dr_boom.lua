local card = {
    api_version = 1, id = "GVG_110", name = "Dr. Boom",
    text = "<b>Battlecry:</b> Summon two 1/1 Boom Bots. <i>WARNING: Bots may explode.</i>",
    set = "GVG", type = "minion", rarity = "legendary", cost = 7, attack = 7, health = 7,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    ctx:summon(player, "GVG_110t")
    ctx:summon(player, "GVG_110t")
end
card.tokens = {{
    id = "GVG_110t", name = "Boom Bot", text = "<b>Deathrattle:</b> Deal 1-4 damage to a random enemy.",
    set = "GVG", type = "minion", cost = 1, attack = 1, health = 1, tags = { "mech" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self) ctx:random_value({ 1, 2, 3, 4 }, "roll_boom_damage") end,
    roll_boom_damage = function(ctx, self, amount)
        ctx:set_data(self, "boom_damage", amount)
        ctx:random_entity(ctx:enemy_characters(self), "explode")
    end,
    explode = function(ctx, self, target) cardlib.effects.damage(ctx, target, ctx:get_data(self, "boom_damage")) end,
}}
return card
