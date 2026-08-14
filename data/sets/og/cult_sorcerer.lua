local function buff_cthun(ctx, player)
    ctx:grant_player_keyword(player, "cthun_buffs")
    ctx:increment_player_data(player, "cthun_attack_buff", 1)
    ctx:increment_player_data(player, "cthun_health_buff", 1)
end

return {
    api_version = 1,
    id = "OG_303",
    name = "Cult Sorcerer",
    text = "[x]<b><b>Spell Damage</b> +1</b>\nAfter you cast a spell,\ngive your C'Thun +1/+1\n<i>(wherever it is).</i>",
    set = "OG",
    type = "minion",
    class = "mage",
    rarity = "rare",
    cost = 2,
    attack = 3,
    health = 2,
    keywords = { "spell_damage" },
    keyword_params = { spell_damage = 1 },
    triggers = {{
        event = "spell_cast",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self)
            buff_cthun(ctx, ctx:controller(self))
        end,
    }},
}
